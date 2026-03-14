use crate::constants::events;
use crate::errors::AppError;
use crate::models::{PhraseSection1Overview, PhraseSection2Context, PhraseSection3Related};
use tauri::{AppHandle, Emitter};

/// Trait for emitting progressive section updates during phrase definition generation.
/// This abstraction allows for testing without Tauri's AppHandle.
pub trait PhraseProgressiveEmitter {
    fn emit_section1(&self, payload: &PhraseSection1Overview) -> Result<(), AppError>;
    fn emit_section2(&self, payload: &PhraseSection2Context) -> Result<(), AppError>;
    fn emit_section3(&self, payload: &PhraseSection3Related) -> Result<(), AppError>;
}

/// Production implementation that emits events via Tauri's AppHandle.
pub struct PhraseAppHandleEmitter<'a> {
    app: &'a AppHandle,
}

impl<'a> PhraseAppHandleEmitter<'a> {
    pub fn new(app: &'a AppHandle) -> Self {
        Self { app }
    }
}

impl<'a> PhraseProgressiveEmitter for PhraseAppHandleEmitter<'a> {
    fn emit_section1(&self, payload: &PhraseSection1Overview) -> Result<(), AppError> {
        self.app
            .emit(events::PHRASE_SECTION_1_OVERVIEW, payload)
            .map_err(|e| AppError::EventEmission(e.to_string()))
    }

    fn emit_section2(&self, payload: &PhraseSection2Context) -> Result<(), AppError> {
        self.app
            .emit(events::PHRASE_SECTION_2_CONTEXT, payload)
            .map_err(|e| AppError::EventEmission(e.to_string()))
    }

    fn emit_section3(&self, payload: &PhraseSection3Related) -> Result<(), AppError> {
        self.app
            .emit(events::PHRASE_SECTION_3_RELATED, payload)
            .map_err(|e| AppError::EventEmission(e.to_string()))
    }
}

#[cfg(test)]
pub mod test_support {
    use super::*;
    use std::cell::RefCell;

    #[derive(Default)]
    pub struct TestPhraseEmitter {
        section1: RefCell<Option<PhraseSection1Overview>>,
        section2: RefCell<Option<PhraseSection2Context>>,
        section3: RefCell<Option<PhraseSection3Related>>,
        emission_order: RefCell<Vec<String>>,
    }

    impl TestPhraseEmitter {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn get_section1(&self) -> Option<PhraseSection1Overview> {
            self.section1.borrow().clone()
        }

        pub fn get_section2(&self) -> Option<PhraseSection2Context> {
            self.section2.borrow().clone()
        }

        pub fn get_section3(&self) -> Option<PhraseSection3Related> {
            self.section3.borrow().clone()
        }

        pub fn get_emission_order(&self) -> Vec<String> {
            self.emission_order.borrow().clone()
        }
    }

    impl PhraseProgressiveEmitter for TestPhraseEmitter {
        fn emit_section1(&self, payload: &PhraseSection1Overview) -> Result<(), AppError> {
            *self.section1.borrow_mut() = Some(payload.clone());
            self.emission_order.borrow_mut().push("section1".to_string());
            Ok(())
        }

        fn emit_section2(&self, payload: &PhraseSection2Context) -> Result<(), AppError> {
            *self.section2.borrow_mut() = Some(payload.clone());
            self.emission_order.borrow_mut().push("section2".to_string());
            Ok(())
        }

        fn emit_section3(&self, payload: &PhraseSection3Related) -> Result<(), AppError> {
            *self.section3.borrow_mut() = Some(payload.clone());
            self.emission_order.borrow_mut().push("section3".to_string());
            Ok(())
        }
    }
}
