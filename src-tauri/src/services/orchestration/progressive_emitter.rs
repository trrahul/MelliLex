use crate::constants::events;
use crate::errors::AppError;
use crate::models::{WordSection1Header, WordSection2Meanings, WordSection3Related};
use tauri::{AppHandle, Emitter};

/// Trait for emitting progressive section updates during word definition generation.
/// This abstraction allows for testing without Tauri's AppHandle.
pub trait ProgressiveEmitter {
    fn emit_section1(&self, payload: &WordSection1Header) -> Result<(), AppError>;
    fn emit_section2(&self, payload: &WordSection2Meanings) -> Result<(), AppError>;
    fn emit_section3(&self, payload: &WordSection3Related) -> Result<(), AppError>;
}

/// Production implementation that emits events via Tauri's AppHandle.
pub struct AppHandleEmitter<'a> {
    app: &'a AppHandle,
}

impl<'a> AppHandleEmitter<'a> {
    pub fn new(app: &'a AppHandle) -> Self {
        Self { app }
    }
}

impl<'a> ProgressiveEmitter for AppHandleEmitter<'a> {
    fn emit_section1(&self, payload: &WordSection1Header) -> Result<(), AppError> {
        self.app
            .emit(events::WORD_SECTION_1_HEADER, payload)
            .map_err(|e| AppError::EventEmission(e.to_string()))
    }

    fn emit_section2(&self, payload: &WordSection2Meanings) -> Result<(), AppError> {
        self.app
            .emit(events::WORD_SECTION_2_MEANINGS, payload)
            .map_err(|e| AppError::EventEmission(e.to_string()))
    }

    fn emit_section3(&self, payload: &WordSection3Related) -> Result<(), AppError> {
        self.app
            .emit(events::WORD_SECTION_3_RELATED, payload)
            .map_err(|e| AppError::EventEmission(e.to_string()))
    }
}

#[cfg(test)]
pub mod test_support {
    use super::*;
    use std::cell::RefCell;

    #[derive(Default)]
    pub struct TestEmitter {
        section1: RefCell<Option<WordSection1Header>>,
        section2: RefCell<Option<WordSection2Meanings>>,
        section3: RefCell<Option<WordSection3Related>>,
        emission_order: RefCell<Vec<String>>,
    }

    impl TestEmitter {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn get_section1(&self) -> Option<WordSection1Header> {
            self.section1.borrow().clone()
        }

        pub fn get_section2(&self) -> Option<WordSection2Meanings> {
            self.section2.borrow().clone()
        }

        pub fn get_section3(&self) -> Option<WordSection3Related> {
            self.section3.borrow().clone()
        }

        pub fn get_emission_order(&self) -> Vec<String> {
            self.emission_order.borrow().clone()
        }
    }

    impl ProgressiveEmitter for TestEmitter {
        fn emit_section1(&self, payload: &WordSection1Header) -> Result<(), AppError> {
            *self.section1.borrow_mut() = Some(payload.clone());
            self.emission_order.borrow_mut().push("section1".to_string());
            Ok(())
        }

        fn emit_section2(&self, payload: &WordSection2Meanings) -> Result<(), AppError> {
            *self.section2.borrow_mut() = Some(payload.clone());
            self.emission_order.borrow_mut().push("section2".to_string());
            Ok(())
        }

        fn emit_section3(&self, payload: &WordSection3Related) -> Result<(), AppError> {
            *self.section3.borrow_mut() = Some(payload.clone());
            self.emission_order.borrow_mut().push("section3".to_string());
            Ok(())
        }
    }
}
