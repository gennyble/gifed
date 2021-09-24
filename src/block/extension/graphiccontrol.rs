use std::{convert::TryInto, fmt};

#[derive(Clone, Debug)]
pub struct GraphicControl {
    pub(crate) packed: u8,
    pub(crate) delay_time: u16,
    pub(crate) transparency_index: u8,
}

impl GraphicControl {
    pub fn new(
        disposal_method: DisposalMethod,
        user_input_flag: bool,
        transparency_flag: bool,
        delay_time: u16,
        transparency_index: u8,
    ) -> Self {
        let mut ret = Self {
            packed: 0,
            delay_time,
            transparency_index,
        };

        ret.set_disposal_method(disposal_method);
        ret.user_input(user_input_flag);
        ret.transparency(transparency_flag);

        ret
    }

    pub fn disposal_method(&self) -> Option<DisposalMethod> {
        match self.packed & 0b000_111_00 {
            0b000_000_00 => Some(DisposalMethod::NoAction),
            0b000_100_00 => Some(DisposalMethod::DoNotDispose),
            0b000_010_00 => Some(DisposalMethod::RestoreBackground),
            0b000_110_00 => Some(DisposalMethod::RestorePrevious),
            _ => None,
        }
    }

    pub fn set_disposal_method(&mut self, method: DisposalMethod) {
        match method {
            DisposalMethod::NoAction => self.packed &= 0b111_000_1_1,
            DisposalMethod::DoNotDispose => self.packed |= 0b000_100_0_0,
            DisposalMethod::RestoreBackground => self.packed |= 0b000_010_0_0,
            DisposalMethod::RestorePrevious => self.packed |= 0b000_110_0_0,
        };
    }

    pub fn transparency_index(&self) -> u8 {
        self.transparency_index
    }

    pub fn user_input(&mut self, flag: bool) {
        if flag {
            self.packed |= 0b000_000_1_0;
        } else {
            self.packed &= 0b111_111_0_1;
        }
    }

    pub fn transparency(&mut self, flag: bool) {
        if flag {
            self.packed |= 0b000_000_0_1;
        } else {
            self.packed &= 0b111_111_1_0;
        }
    }

    pub fn delay_time(&self) -> u16 {
        self.delay_time
    }

    pub fn delay_time_mut(&mut self) -> &mut u16 {
        &mut self.delay_time
    }

    //TODO: Transparency index setter
}

impl From<[u8; 4]> for GraphicControl {
    fn from(arr: [u8; 4]) -> Self {
        let packed = arr[0];
        let delay_time = u16::from_le_bytes(arr[1..3].try_into().unwrap());
        let transparency_index = arr[3];

        Self {
            packed,
            delay_time,
            transparency_index,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DisposalMethod {
    NoAction,
    DoNotDispose,
    RestoreBackground,
    RestorePrevious,
}

impl fmt::Display for DisposalMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let st = match self {
            DisposalMethod::NoAction => "Dispose as Normal",
            DisposalMethod::DoNotDispose => "No Dispose",
            DisposalMethod::RestoreBackground => "Restore to background",
            DisposalMethod::RestorePrevious => "Restore previous image",
        };

        write!(f, "{}", st)
    }
}
