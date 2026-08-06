use core::sync::atomic::{AtomicU8, Ordering::Relaxed};

use embassy_stm32::{
    Peri,
    adc::{Adc, AnyAdcChannel, SampleTime},
    dma::InterruptHandler,
    gpio::Output,
    interrupt::{self, typelevel::DMA1_CHANNEL1},
    peripherals::{ADC1, DMA1_CH1},
};
use rmk::{
    core_traits::Runnable,
    embassy_time::Timer,
    event::{KeyboardEvent, publish_event_async},
};

pub struct ApexOpticalMatrix<
    'd,
    I: interrupt::typelevel::Binding<DMA1_CHANNEL1, InterruptHandler<DMA1_CH1>>,
    const ROW: usize,
    const COL: usize,
> {
    row_outputs: [Output<'d>; ROW],
    col_inputs: [AnyAdcChannel<'d, ADC1>; COL],
    adc: Adc<'d, ADC1>,
    dma: Peri<'d, DMA1_CH1>,
    irq: I,

    scratch: [u16; COL],
    min_value: [[u16; COL]; ROW],
    max_value: [[u16; COL]; ROW],
    was_triggered: [[bool; COL]; ROW],

    thresholds: &'d Thresholds,
}

impl<
    'd,
    I: interrupt::typelevel::Binding<DMA1_CHANNEL1, InterruptHandler<DMA1_CH1>>,
    const ROW: usize,
    const COL: usize,
> ApexOpticalMatrix<'d, I, ROW, COL>
{
    pub fn new(
        adc: Peri<'d, ADC1>,
        dma: Peri<'d, DMA1_CH1>,
        irq: I,
        col_inputs: [AnyAdcChannel<'d, ADC1>; COL],
        row_outputs: [Output<'d>; ROW],
        thresholds: &'d Thresholds,
    ) -> Self {
        let adc = Adc::new(adc);

        Self {
            col_inputs,
            row_outputs,
            adc,
            dma,
            irq,

            scratch: [0u16; COL],
            min_value: [[512u16; COL]; ROW],
            max_value: [[2048u16; COL]; ROW],
            was_triggered: [[false; COL]; ROW],

            thresholds,
        }
    }

    async fn scan(&mut self) {
        let thresholds = self.thresholds.tester();

        for (row_idx, ((mins, maxs), was_triggereds)) in self
            .min_value
            .iter_mut()
            .zip(&mut self.max_value)
            .zip(&mut self.was_triggered)
            .enumerate()
        {
            self.row_outputs[row_idx].set_low();
            Timer::after_micros(20).await;

            self.adc
                .read(
                    self.dma.reborrow(),
                    self.irq,
                    self.col_inputs
                        .iter_mut()
                        .map(|a| (a, SampleTime::CYCLES92_5)),
                    &mut self.scratch,
                )
                .await;
            self.row_outputs[row_idx].set_high();
            if row_idx < ROW - 1 {
                self.row_outputs[row_idx + 1].set_low();
            }

            for (col_idx, (((val, min), max), was_triggered)) in self
                .scratch
                .iter()
                .zip(mins.iter_mut())
                .zip(maxs.iter_mut())
                .zip(was_triggereds.iter_mut())
                .enumerate()
            {
                *min = (*min).min(*val);
                *max = (*max).max(*val);

                let is_triggered = thresholds.should_be_triggered(*val, *min, *max, *was_triggered);
                if is_triggered != *was_triggered {
                    publish_event_async(KeyboardEvent::key(
                        row_idx as u8,
                        col_idx as u8,
                        is_triggered,
                    ))
                    .await;
                }
                *was_triggered = is_triggered;
            }
        }
    }

    pub async fn check_key(&mut self, row: usize, col: usize) -> bool {
        assert!(row < ROW);
        assert!(col < COL);

        self.row_outputs[row].set_low();
        Timer::after_micros(20).await;
        let mut val = [0u16; 1];
        self.adc
            .read(
                self.dma.reborrow(),
                self.irq,
                [(&mut self.col_inputs[col], SampleTime::CYCLES92_5)].into_iter(),
                &mut val,
            )
            .await;
        let val = val[0];

        let min = self.min_value[row][col];
        let max = self.max_value[row][col];

        let triggered = self
            .thresholds
            .tester()
            .should_be_triggered(val, min, max, false);

        self.row_outputs[row].set_high();

        triggered
    }
}

impl<
    'd,
    I: interrupt::typelevel::Binding<DMA1_CHANNEL1, InterruptHandler<DMA1_CH1>>,
    const ROW: usize,
    const COL: usize,
> Runnable for ApexOpticalMatrix<'d, I, ROW, COL>
{
    async fn run(&mut self) -> ! {
        loop {
            self.scan().await;
        }
    }
}

pub struct Thresholds {
    pub thres_trigger: AtomicU8,
    pub thres_untrigger: AtomicU8,
}

impl Thresholds {
    fn tester(&self) -> ThresholdTester {
        ThresholdTester {
            thres_trigger: self.thres_trigger.load(Relaxed),
            thres_untrigger: self.thres_untrigger.load(Relaxed),
        }
    }
}

struct ThresholdTester {
    thres_trigger: u8,
    thres_untrigger: u8,
}

impl ThresholdTester {
    fn should_be_triggered(&self, val: u16, min: u16, max: u16, was_triggered: bool) -> bool {
        if val < min {
            return false;
        }

        let range = (max - min) as u32;
        let ratio = ((val as u32) * 0xFF / range).min(0xFF) as u8;

        if ratio > self.thres_trigger {
            true
        } else if ratio < self.thres_untrigger {
            false
        } else {
            was_triggered
        }
    }
}
