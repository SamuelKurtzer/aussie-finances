use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ExpenseFrequency {
    Weekly,
    Fortnightly,
    #[default]
    Monthly,
    Quarterly,
    Annual,
}

impl ExpenseFrequency {
    pub const ALL: [ExpenseFrequency; 5] = [
        ExpenseFrequency::Weekly,
        ExpenseFrequency::Fortnightly,
        ExpenseFrequency::Monthly,
        ExpenseFrequency::Quarterly,
        ExpenseFrequency::Annual,
    ];

    pub fn occurrences_per_year(self) -> f64 {
        match self {
            ExpenseFrequency::Weekly => 52.0,
            ExpenseFrequency::Fortnightly => 26.0,
            ExpenseFrequency::Monthly => 12.0,
            ExpenseFrequency::Quarterly => 4.0,
            ExpenseFrequency::Annual => 1.0,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ExpenseFrequency::Weekly => "Weekly",
            ExpenseFrequency::Fortnightly => "Fortnightly",
            ExpenseFrequency::Monthly => "Monthly",
            ExpenseFrequency::Quarterly => "Quarterly",
            ExpenseFrequency::Annual => "Annual",
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            ExpenseFrequency::Weekly => "weekly",
            ExpenseFrequency::Fortnightly => "fortnightly",
            ExpenseFrequency::Monthly => "monthly",
            ExpenseFrequency::Quarterly => "quarterly",
            ExpenseFrequency::Annual => "annual",
        }
    }

    pub fn from_key(key: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|f| f.key() == key)
            .unwrap_or_default()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExpenseItem {
    pub id: u32,
    pub name: String,
    pub amount: f64,
    pub frequency: ExpenseFrequency,
}

impl ExpenseItem {
    pub fn annual_amount(&self) -> f64 {
        self.amount * self.frequency.occurrences_per_year()
    }

    pub fn monthly_amount(&self) -> f64 {
        self.annual_amount() / 12.0
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct BudgetInput {
    pub items: Vec<ExpenseItem>,
}

impl BudgetInput {
    pub fn next_id(&self) -> u32 {
        self.items.iter().map(|i| i.id).max().unwrap_or(0) + 1
    }

    pub fn monthly_total(&self) -> f64 {
        self.annual_total() / 12.0
    }

    pub fn annual_total(&self) -> f64 {
        self.items.iter().map(|i| i.annual_amount()).sum()
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    fn item(amount: f64, frequency: ExpenseFrequency) -> ExpenseItem {
        ExpenseItem {
            id: 1,
            name: "Test".to_string(),
            amount,
            frequency,
        }
    }

    #[test]
    fn frequencies_convert_to_annual() {
        assert_relative_eq!(
            item(100.0, ExpenseFrequency::Weekly).annual_amount(),
            5_200.0
        );
        assert_relative_eq!(
            item(100.0, ExpenseFrequency::Fortnightly).annual_amount(),
            2_600.0
        );
        assert_relative_eq!(
            item(100.0, ExpenseFrequency::Monthly).annual_amount(),
            1_200.0
        );
        assert_relative_eq!(
            item(100.0, ExpenseFrequency::Quarterly).annual_amount(),
            400.0
        );
        assert_relative_eq!(item(100.0, ExpenseFrequency::Annual).annual_amount(), 100.0);
    }

    #[test]
    fn totals_sum_across_items() {
        let budget = BudgetInput {
            items: vec![
                item(300.0, ExpenseFrequency::Weekly),
                ExpenseItem {
                    id: 2,
                    name: "Insurance".to_string(),
                    amount: 1_200.0,
                    frequency: ExpenseFrequency::Annual,
                },
            ],
        };
        assert_relative_eq!(budget.annual_total(), 300.0 * 52.0 + 1_200.0);
        assert_relative_eq!(budget.monthly_total(), (300.0 * 52.0 + 1_200.0) / 12.0);
    }

    #[test]
    fn next_id_increments_past_max() {
        let mut budget = BudgetInput::default();
        assert_eq!(budget.next_id(), 1);
        budget.items.push(item(10.0, ExpenseFrequency::Monthly));
        budget.items.push(ExpenseItem {
            id: 7,
            name: "Gap".to_string(),
            amount: 10.0,
            frequency: ExpenseFrequency::Monthly,
        });
        assert_eq!(budget.next_id(), 8);
    }

    #[test]
    fn old_storage_format_missing_items_still_loads() {
        let budget: BudgetInput = serde_json::from_str("{}").unwrap();
        assert!(budget.items.is_empty());
    }
}
