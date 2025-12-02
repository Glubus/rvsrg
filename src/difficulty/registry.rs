//! Registry for managing multiple difficulty calculators.
//!
//! The registry holds all available calculators (built-in and custom) and
//! provides methods to calculate difficulty using a specific calculator.

use super::builtin::{EtternaCalculator, OsuCalculator};
use super::calculator::{CalcError, CalculationContext, DifficultyCalculator};
use super::BeatmapSsr;
use std::collections::HashMap;
use std::sync::Arc;

/// Registry holding all available difficulty calculators.
#[derive(Debug)]
pub struct CalculatorRegistry {
    calculators: HashMap<String, Arc<dyn DifficultyCalculator>>,
    default_calculator: String,
}

impl Default for CalculatorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl CalculatorRegistry {
    /// Creates a new registry with built-in calculators.
    pub fn new() -> Self {
        let mut registry = Self {
            calculators: HashMap::new(),
            default_calculator: "etterna".to_string(),
        };

        // Register built-in calculators
        registry.register(Arc::new(EtternaCalculator::new()));
        registry.register(Arc::new(OsuCalculator::new()));

        registry
    }

    /// Registers a new calculator.
    pub fn register(&mut self, calculator: Arc<dyn DifficultyCalculator>) {
        let id = calculator.id().to_string();
        self.calculators.insert(id, calculator);
    }

    /// Returns the calculator with the given ID.
    pub fn get(&self, id: &str) -> Option<&Arc<dyn DifficultyCalculator>> {
        self.calculators.get(id)
    }

    /// Returns the default calculator.
    pub fn default_calculator(&self) -> Option<&Arc<dyn DifficultyCalculator>> {
        self.calculators.get(&self.default_calculator)
    }

    /// Sets the default calculator ID.
    pub fn set_default(&mut self, id: &str) {
        if self.calculators.contains_key(id) {
            self.default_calculator = id.to_string();
        }
    }

    /// Returns all registered calculator IDs.
    pub fn calculator_ids(&self) -> Vec<&str> {
        self.calculators.keys().map(|s| s.as_str()).collect()
    }

    /// Returns all registered calculators with their display names.
    pub fn calculators_with_names(&self) -> Vec<(&str, &str)> {
        self.calculators
            .iter()
            .map(|(id, calc)| (id.as_str(), calc.display_name()))
            .collect()
    }

    /// Calculates difficulty using the specified calculator.
    pub fn calculate(
        &self,
        calculator_id: &str,
        ctx: &CalculationContext,
    ) -> Result<BeatmapSsr, CalcError> {
        let calculator = self
            .calculators
            .get(calculator_id)
            .ok_or_else(|| CalcError::Other(format!("Calculator '{}' not found", calculator_id)))?;

        calculator.calculate(ctx)
    }

    /// Calculates difficulty using the default calculator.
    pub fn calculate_default(&self, ctx: &CalculationContext) -> Result<BeatmapSsr, CalcError> {
        self.calculate(&self.default_calculator, ctx)
    }

    /// Returns the available rates for a calculator.
    pub fn available_rates(&self, calculator_id: &str) -> Option<Vec<f64>> {
        self.calculators
            .get(calculator_id)
            .and_then(|calc| calc.available_rates())
    }

    /// Checks if a calculator supports arbitrary rates.
    pub fn supports_arbitrary_rates(&self, calculator_id: &str) -> bool {
        self.calculators
            .get(calculator_id)
            .map(|calc| calc.supports_arbitrary_rates())
            .unwrap_or(false)
    }
}

/// Global calculator registry singleton.
static REGISTRY: std::sync::OnceLock<std::sync::RwLock<CalculatorRegistry>> =
    std::sync::OnceLock::new();

/// Gets a reference to the global calculator registry.
pub fn global_registry() -> &'static std::sync::RwLock<CalculatorRegistry> {
    REGISTRY.get_or_init(|| std::sync::RwLock::new(CalculatorRegistry::new()))
}

/// Calculates difficulty using the global registry.
pub fn calculate_with_registry(
    calculator_id: &str,
    ctx: &CalculationContext,
) -> Result<BeatmapSsr, CalcError> {
    global_registry()
        .read()
        .map_err(|_| CalcError::Other("Registry lock poisoned".to_string()))?
        .calculate(calculator_id, ctx)
}

