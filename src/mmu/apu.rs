#[derive(Default)]
struct ChannelOne {
    sweep: SweepReg,
    ln_timer_duty_cycle: LnTimerDutyCycleReg,
    volume_envelope: VolumeEnvReg,
    period_high_ctrl: PeriodHighCtrlReg,
    period_low: PeriodLowReg,
}

#[derive(Default)]
struct ChannelTwo {
    channel_two_ln_timer_duty_cycle: LnTimerDutyCycleReg,
    channel_two_volume_envelope: VolumeEnvReg,
    channel_two_period_high_ctrl: PeriodHighCtrlReg,
    channel_two_period_low: PeriodLowReg,
}

#[derive(Default)]
struct ChannelThree {
    channel_three_dac_enable: WaveDacEnableReg,
    channel_three_ln_timer: WaveLengthTimerReg,
    channel_three_output_level: OutputLevelReg,
    channel_three_period_low: PeriodLowReg,
    channel_three_period_high_crtl: PeriodHighCtrlReg,
}

#[derive(Default)]
struct ChannelFour {
    channel_four_length_timer: NoiseLengthTimer,
    channel_four_volume_evelope: VolumeEnvReg,
    channel_four_freq_and_rand: FreqRandomnessReg,
    channel_four_ctrl: ChannelFourCtrlReg,
}

#[derive(Default)]
pub struct Apu {
    audio_master_control: AudioMasterControlReg,
    sound_panning: SoundPanningReg,
    master_vol_and_vin_panning: MasterVolVinPanningReg,

    channel_one: ChannelOne,
    channel_two: ChannelTwo,
    channel_three: ChannelThree,
    channel_four: ChannelFour,
}

trait Channel {}

macro_rules! define_register {
    ($name:ident) => {
        #[derive(Default, Debug, Copy, Clone)]
        struct $name { byte: u8, }
    };
}

macro_rules! read_write_register {
    ($name:ident) => {
        define_register!($name);
        impl Register for $name {
            fn read(&self) -> u8 { self.byte }
            fn write(&mut self, value: u8) { self.byte = value}
        }
    };
}

macro_rules! write_only_register {
    ($name:ident) => {
        define_register!($name);
        impl Register for $name {
            fn read(&self) -> u8 { 0xFF }
            fn write(&mut self, value: u8) { self.byte = value}
        }
    };
}

read_write_register!(AudioMasterControlReg);
read_write_register!(SoundPanningReg);
read_write_register!(MasterVolVinPanningReg);
read_write_register!(SweepReg);

define_register!(LnTimerDutyCycleReg);
impl Register for LnTimerDutyCycleReg {
    fn read(&self) -> u8 { (self.byte & 0b1100_0000) | 0b0011_1111 }
    fn write(&mut self, value: u8) { self.byte = value;}
}

read_write_register!(VolumeEnvReg);
write_only_register!(PeriodHighCtrlReg);
write_only_register!(PeriodLowReg);
read_write_register!(WaveDacEnableReg);
write_only_register!(WaveLengthTimerReg);

read_write_register!(OutputLevelReg);
read_write_register!(NoiseLengthTimer);
read_write_register!(FreqRandomnessReg);
define_register!(ChannelFourCtrlReg);
impl Register for ChannelFourCtrlReg {
    fn read(&self) -> u8 { (self.byte & 0b0110_0000) | 0b1001_1111 }
    fn write(&mut self, value: u8) { self.byte = value }
}

trait Register {
    fn read(&self) -> u8;
    fn write(&mut self, value: u8);
}

impl Apu {
    pub fn read(&self, addr: u16) -> u8 { 0xFF }
    pub fn write(&self, addr:u16 , value :u8) { }
}
