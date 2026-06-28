use std::collections::HashSet;
use std::time::{Duration, Instant};

use crate::app::actions::AppAction;
use crate::config::{AppConfig, KeyName};
use crate::core::GameMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HorizontalDirection {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RepeatKeyState {
    pub pressed: bool,
    pub pressed_at: Instant,
    pub next_repeat_at: Instant,
}

impl Default for RepeatKeyState {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            pressed: false,
            pressed_at: now,
            next_repeat_at: now,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InputRepeatState {
    pub left: RepeatKeyState,
    pub right: RepeatKeyState,
    pub soft_drop: RepeatKeyState,
    pub active_horizontal: Option<HorizontalDirection>,
}

impl InputRepeatState {
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn horizontal_mut(&mut self, direction: HorizontalDirection) -> &mut RepeatKeyState {
        match direction {
            HorizontalDirection::Left => &mut self.left,
            HorizontalDirection::Right => &mut self.right,
        }
    }

    pub fn press_horizontal(
        &mut self,
        direction: HorizontalDirection,
        now: Instant,
        das_ms: u32,
        das_cancel: bool,
    ) {
        let opposite_pressed = self.horizontal(direction.opposite()).pressed;
        let key_state = self.horizontal_mut(direction);
        key_state.pressed = true;
        key_state.pressed_at = now;
        key_state.next_repeat_at = now + Duration::from_millis(u64::from(das_ms));

        self.active_horizontal = if das_cancel || !opposite_pressed {
            Some(direction)
        } else {
            None
        };
    }

    pub fn release_horizontal(&mut self, direction: HorizontalDirection, das_ms: u32) {
        let released = *self.horizontal(direction);
        self.horizontal_mut(direction).pressed = false;

        if self.active_horizontal == Some(direction) {
            let opposite = direction.opposite();
            if self.horizontal(opposite).pressed {
                let opposite_state = self.horizontal_mut(opposite);
                opposite_state.pressed_at = released.pressed_at;
                opposite_state.next_repeat_at =
                    released.pressed_at + Duration::from_millis(u64::from(das_ms));
                self.active_horizontal = Some(opposite);
            } else {
                self.active_horizontal = None;
            }
        } else if !self.left.pressed && !self.right.pressed {
            self.active_horizontal = None;
        }
    }

    pub fn press_soft_drop(&mut self, now: Instant, sdd_ms: u32) {
        self.soft_drop.pressed = true;
        self.soft_drop.pressed_at = now;
        self.soft_drop.next_repeat_at = now + Duration::from_millis(u64::from(sdd_ms));
    }

    pub fn release_soft_drop(&mut self) {
        self.soft_drop.pressed = false;
    }

    fn horizontal(&self, direction: HorizontalDirection) -> &RepeatKeyState {
        match direction {
            HorizontalDirection::Left => &self.left,
            HorizontalDirection::Right => &self.right,
        }
    }
}

impl HorizontalDirection {
    fn opposite(self) -> Self {
        match self {
            HorizontalDirection::Left => HorizontalDirection::Right,
            HorizontalDirection::Right => HorizontalDirection::Left,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Trigger {
    Press,
    Release,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShiftRequirement {
    Any,
    NotPressed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CtrlRequirement {
    Any,
    NotPressed,
    Pressed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BindingAction {
    TogglePause,
    Reset(GameMode),
    ResetCurrent,
    RequestScreenshot,
    MoveLeftPress,
    MoveLeftRelease,
    MoveRightPress,
    MoveRightRelease,
    SoftDropPress,
    SoftDropRelease,
    HardDrop,
    RotateCcw,
    RotateCw,
    Rotate180,
    Hold,
    Undo,
    Redo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct KeyBinding {
    key_name: KeyName,
    trigger: Trigger,
    shift: ShiftRequirement,
    ctrl: CtrlRequirement,
    action: BindingAction,
}

const APP_KEY_BINDINGS: [KeyBinding; 8] = [
    KeyBinding {
        key_name: KeyName::Escape,
        trigger: Trigger::Press,
        shift: ShiftRequirement::Any,
        ctrl: CtrlRequirement::Any,
        action: BindingAction::TogglePause,
    },
    KeyBinding {
        key_name: KeyName::F1,
        trigger: Trigger::Press,
        shift: ShiftRequirement::Any,
        ctrl: CtrlRequirement::Any,
        action: BindingAction::Reset(GameMode::Training),
    },
    KeyBinding {
        key_name: KeyName::F2,
        trigger: Trigger::Press,
        shift: ShiftRequirement::Any,
        ctrl: CtrlRequirement::Any,
        action: BindingAction::Reset(GameMode::Cheese),
    },
    KeyBinding {
        key_name: KeyName::F3,
        trigger: Trigger::Press,
        shift: ShiftRequirement::Any,
        ctrl: CtrlRequirement::Any,
        action: BindingAction::Reset(GameMode::FourWide),
    },
    KeyBinding {
        key_name: KeyName::F4,
        trigger: Trigger::Press,
        shift: ShiftRequirement::Any,
        ctrl: CtrlRequirement::Any,
        action: BindingAction::Reset(GameMode::PerfectClear),
    },
    KeyBinding {
        key_name: KeyName::F5,
        trigger: Trigger::Press,
        shift: ShiftRequirement::Any,
        ctrl: CtrlRequirement::Any,
        action: BindingAction::Reset(GameMode::Master),
    },
    KeyBinding {
        key_name: KeyName::F6,
        trigger: Trigger::Press,
        shift: ShiftRequirement::Any,
        ctrl: CtrlRequirement::Any,
        action: BindingAction::RequestScreenshot,
    },
    KeyBinding {
        key_name: KeyName::R,
        trigger: Trigger::Press,
        shift: ShiftRequirement::Any,
        ctrl: CtrlRequirement::Any,
        action: BindingAction::ResetCurrent,
    },
];

pub fn collect_keyboard_actions(
    ctx: &egui::Context,
    now: Instant,
    config: &AppConfig,
    actions: &mut Vec<AppAction>,
    previous_keys: &mut HashSet<egui::Key>,
) {
    ctx.input(|input| {
        let keys_down = &input.keys_down;

        for binding in APP_KEY_BINDINGS {
            collect_binding_action(input, keys_down, previous_keys, binding, now, actions);
        }

        let bindings = &config.bindings;
        let mut gameplay_bindings = vec![
            KeyBinding {
                key_name: bindings.move_left,
                trigger: Trigger::Press,
                shift: ShiftRequirement::Any,
                ctrl: CtrlRequirement::Any,
                action: BindingAction::MoveLeftPress,
            },
            KeyBinding {
                key_name: bindings.move_left,
                trigger: Trigger::Release,
                shift: ShiftRequirement::Any,
                ctrl: CtrlRequirement::Any,
                action: BindingAction::MoveLeftRelease,
            },
            KeyBinding {
                key_name: bindings.move_right,
                trigger: Trigger::Press,
                shift: ShiftRequirement::Any,
                ctrl: CtrlRequirement::Any,
                action: BindingAction::MoveRightPress,
            },
            KeyBinding {
                key_name: bindings.move_right,
                trigger: Trigger::Release,
                shift: ShiftRequirement::Any,
                ctrl: CtrlRequirement::Any,
                action: BindingAction::MoveRightRelease,
            },
            KeyBinding {
                key_name: bindings.soft_drop,
                trigger: Trigger::Press,
                shift: ShiftRequirement::Any,
                ctrl: CtrlRequirement::Any,
                action: BindingAction::SoftDropPress,
            },
            KeyBinding {
                key_name: bindings.soft_drop,
                trigger: Trigger::Release,
                shift: ShiftRequirement::Any,
                ctrl: CtrlRequirement::Any,
                action: BindingAction::SoftDropRelease,
            },
            KeyBinding {
                key_name: bindings.rotate_ccw,
                trigger: Trigger::Press,
                shift: ShiftRequirement::NotPressed,
                ctrl: CtrlRequirement::NotPressed,
                action: BindingAction::RotateCcw,
            },
            KeyBinding {
                key_name: bindings.rotate_cw,
                trigger: Trigger::Press,
                shift: ShiftRequirement::Any,
                ctrl: CtrlRequirement::NotPressed,
                action: BindingAction::RotateCw,
            },
            KeyBinding {
                key_name: bindings.rotate_180,
                trigger: Trigger::Press,
                shift: ShiftRequirement::Any,
                ctrl: CtrlRequirement::NotPressed,
                action: BindingAction::Rotate180,
            },
            KeyBinding {
                key_name: bindings.undo,
                trigger: Trigger::Press,
                shift: ShiftRequirement::Any,
                ctrl: CtrlRequirement::Pressed,
                action: BindingAction::Undo,
            },
            KeyBinding {
                key_name: bindings.redo,
                trigger: Trigger::Press,
                shift: ShiftRequirement::Any,
                ctrl: CtrlRequirement::Pressed,
                action: BindingAction::Redo,
            },
        ];

        if !bindings.hold_with_shift {
            gameplay_bindings.push(KeyBinding {
                key_name: bindings.hold,
                trigger: Trigger::Press,
                shift: ShiftRequirement::Any,
                ctrl: CtrlRequirement::Any,
                action: BindingAction::Hold,
            });
        }

        for binding in gameplay_bindings {
            collect_binding_action(input, keys_down, previous_keys, binding, now, actions);
        }

        for &hard_drop_key in &bindings.hard_drop {
            collect_binding_action(
                input,
                keys_down,
                previous_keys,
                KeyBinding {
                    key_name: hard_drop_key,
                    trigger: Trigger::Press,
                    shift: ShiftRequirement::Any,
                    ctrl: CtrlRequirement::Any,
                    action: BindingAction::HardDrop,
                },
                now,
                actions,
            );
        }

        *previous_keys = keys_down.clone();
    });
}

fn collect_binding_action(
    input: &egui::InputState,
    keys_down: &HashSet<egui::Key>,
    previous_keys: &HashSet<egui::Key>,
    binding: KeyBinding,
    now: Instant,
    actions: &mut Vec<AppAction>,
) {
    if !shift_matches(binding.shift, input.modifiers.shift)
        || !ctrl_matches(binding.ctrl, input.modifiers.ctrl)
    {
        return;
    }

    let key = binding.key_name.egui_key();
    let key_down = keys_down.contains(&key);
    let was_down = previous_keys.contains(&key);

    let triggered = match binding.trigger {
        Trigger::Press => key_down && !was_down,
        Trigger::Release => !key_down && was_down,
    };

    if triggered {
        actions.push(binding.action.into_action(now));
    }
}

fn shift_matches(requirement: ShiftRequirement, shift_pressed: bool) -> bool {
    match requirement {
        ShiftRequirement::Any => true,
        ShiftRequirement::NotPressed => !shift_pressed,
    }
}

fn ctrl_matches(requirement: CtrlRequirement, ctrl_pressed: bool) -> bool {
    match requirement {
        CtrlRequirement::Any => true,
        CtrlRequirement::NotPressed => !ctrl_pressed,
        CtrlRequirement::Pressed => ctrl_pressed,
    }
}

impl BindingAction {
    fn into_action(self, now: Instant) -> AppAction {
        match self {
            BindingAction::TogglePause => AppAction::TogglePause,
            BindingAction::Reset(mode) => AppAction::Reset(mode),
            BindingAction::ResetCurrent => AppAction::ResetCurrent,
            BindingAction::RequestScreenshot => AppAction::RequestScreenshot,
            BindingAction::MoveLeftPress => AppAction::MoveLeftPress(now),
            BindingAction::MoveLeftRelease => AppAction::MoveLeftRelease,
            BindingAction::MoveRightPress => AppAction::MoveRightPress(now),
            BindingAction::MoveRightRelease => AppAction::MoveRightRelease,
            BindingAction::SoftDropPress => AppAction::SoftDropPress(now),
            BindingAction::SoftDropRelease => AppAction::SoftDropRelease,
            BindingAction::HardDrop => AppAction::HardDrop,
            BindingAction::RotateCcw => AppAction::RotateCcw,
            BindingAction::RotateCw => AppAction::RotateCw,
            BindingAction::Rotate180 => AppAction::Rotate180,
            BindingAction::Hold => AppAction::Hold,
            BindingAction::Undo => AppAction::Undo,
            BindingAction::Redo => AppAction::Redo,
        }
    }
}
