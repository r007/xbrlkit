//! # US-GAAP financial statements
//!
//! The three primary statements, as the line items most filers report. Read
//! them all at once with [`Financials`], or any one on its own:
//!
//! ```no_run
//! use xbrlkit::Document;
//! use xbrlkit::taxonomies::us_gaap::{Financials, IncomeStatement};
//!
//! let doc = Document::parse(&std::fs::read_to_string("aapl-20250927.htm")?)?;
//!
//! let financials: Financials = doc.extract()?;
//! println!("total assets: {:?}", financials.balance_sheet.assets);
//!
//! // One income statement per period the filing reports: in a 10-Q, the
//! // quarter and the year to date, each beside last year's.
//! for statement in &financials.income_statements {
//!     println!(
//!         "{:?} to {:?}: revenue {:?}",
//!         statement.period_start, statement.period_end, statement.revenue
//!     );
//! }
//!
//! let (income, problems) = doc.extract_lenient::<IncomeStatement>();
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## What these are, and are not
//!
//! US-GAAP has some 17,000 concepts and lets a filer pick among near
//! synonyms for the same line: revenue alone is reported under half a dozen.
//! Each field here names the concepts it accepts, most common first, and
//! takes the first one the filing reports. That covers the face of the
//! statements for most commercial companies. It does not cover everything a
//! bank, an insurer or a REIT puts on theirs, and it never will hold every
//! line of anyone's.
//!
//! When a field you need is missing, declare your own struct: the derive is
//! the API, and these are forty lines of it. They are also `Serialize` and
//! `Deserialize` under their Rust names, so they go to JSON or a dataframe
//! as they are.

use crate::FromXbrl;
use crate::bind::Document;
use crate::error::Result;
use serde::{Deserialize, Serialize};

/// A balance sheet: what the entity owns and owes at one date.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, FromXbrl)]
#[serde(default)]
#[xbrl(instant)]
pub struct BalanceSheet {
    /// The date these figures are as of.
    ///
    /// Set on each of [`Financials::balance_sheets`]. `None` on
    /// [`Financials::balance_sheet`], whose fields are each the best the
    /// filing offers and can come from different dates.
    #[xbrl(period_end)]
    pub as_of: Option<String>,

    /// Cash and cash equivalents.
    #[xbrl(
        concept = "us-gaap:CashAndCashEquivalentsAtCarryingValue",
        alias = "us-gaap:CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalents",
        alias = "us-gaap:Cash"
    )]
    pub cash_and_cash_equivalents: Option<f64>,

    /// Short-term investments and marketable securities.
    #[xbrl(
        concept = "us-gaap:MarketableSecuritiesCurrent",
        alias = "us-gaap:ShortTermInvestments",
        alias = "us-gaap:AvailableForSaleSecuritiesDebtSecuritiesCurrent"
    )]
    pub short_term_investments: Option<f64>,

    /// Trade receivables, net of allowances.
    #[xbrl(
        concept = "us-gaap:AccountsReceivableNetCurrent",
        alias = "us-gaap:ReceivablesNetCurrent"
    )]
    pub accounts_receivable: Option<f64>,

    /// Inventory.
    #[xbrl(concept = "us-gaap:InventoryNet")]
    pub inventory: Option<f64>,

    /// Total current assets. Absent for filers with an unclassified balance
    /// sheet, such as banks.
    #[xbrl(concept = "us-gaap:AssetsCurrent")]
    pub assets_current: Option<f64>,

    /// Property, plant and equipment, net of depreciation.
    #[xbrl(
        concept = "us-gaap:PropertyPlantAndEquipmentNet",
        alias = "us-gaap:PropertyPlantAndEquipmentAndFinanceLeaseRightOfUseAssetAfterAccumulatedDepreciationAndAmortization"
    )]
    pub property_plant_and_equipment: Option<f64>,

    /// Goodwill.
    #[xbrl(concept = "us-gaap:Goodwill")]
    pub goodwill: Option<f64>,

    /// Intangible assets other than goodwill, net of amortization.
    #[xbrl(
        concept = "us-gaap:IntangibleAssetsNetExcludingGoodwill",
        alias = "us-gaap:FiniteLivedIntangibleAssetsNet"
    )]
    pub intangible_assets: Option<f64>,

    /// Total assets.
    #[xbrl(concept = "us-gaap:Assets")]
    pub assets: Option<f64>,

    /// Trade payables.
    #[xbrl(
        concept = "us-gaap:AccountsPayableCurrent",
        alias = "us-gaap:AccountsPayableTradeCurrent"
    )]
    pub accounts_payable: Option<f64>,

    /// Total current liabilities.
    #[xbrl(concept = "us-gaap:LiabilitiesCurrent")]
    pub liabilities_current: Option<f64>,

    /// Long-term debt, excluding the portion due within a year.
    #[xbrl(
        concept = "us-gaap:LongTermDebtNoncurrent",
        alias = "us-gaap:LongTermDebtAndCapitalLeaseObligations",
        alias = "us-gaap:LongTermDebt"
    )]
    pub long_term_debt: Option<f64>,

    /// Total liabilities. Some filers report only the total of liabilities
    /// and equity, and leave this to subtraction.
    #[xbrl(concept = "us-gaap:Liabilities")]
    pub liabilities: Option<f64>,

    /// Retained earnings, or the accumulated deficit as a negative.
    #[xbrl(concept = "us-gaap:RetainedEarningsAccumulatedDeficit")]
    pub retained_earnings: Option<f64>,

    /// Equity attributable to the parent's shareholders.
    #[xbrl(concept = "us-gaap:StockholdersEquity")]
    pub stockholders_equity: Option<f64>,

    /// Equity attributable to noncontrolling interests.
    #[xbrl(concept = "us-gaap:MinorityInterest")]
    pub noncontrolling_interest: Option<f64>,

    /// Total equity, noncontrolling interests included. The same as
    /// `stockholders_equity` for a filer with none.
    #[xbrl(
        concept = "us-gaap:StockholdersEquityIncludingPortionAttributableToNoncontrollingInterest",
        alias = "us-gaap:StockholdersEquity"
    )]
    pub total_equity: Option<f64>,

    /// Total liabilities and equity. Equals total assets.
    #[xbrl(concept = "us-gaap:LiabilitiesAndStockholdersEquity")]
    pub liabilities_and_equity: Option<f64>,
}

/// An income statement: what the entity earned and spent over one period.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, FromXbrl)]
#[serde(default)]
#[xbrl(duration)]
pub struct IncomeStatement {
    /// First day of the period these figures cover.
    ///
    /// Set, with `period_end`, on each of [`Financials::income_statements`].
    /// `None` on [`Financials::income_statement`], whose fields are each the
    /// best the filing offers and can come from different periods.
    #[xbrl(period_start)]
    pub period_start: Option<String>,

    /// Last day of the period these figures cover.
    #[xbrl(period_end)]
    pub period_end: Option<String>,

    /// Revenue.
    #[xbrl(
        concept = "us-gaap:RevenueFromContractWithCustomerExcludingAssessedTax",
        alias = "us-gaap:Revenues",
        alias = "us-gaap:RevenueFromContractWithCustomerIncludingAssessedTax",
        alias = "us-gaap:SalesRevenueNet"
    )]
    pub revenue: Option<f64>,

    /// Cost of revenue.
    #[xbrl(
        concept = "us-gaap:CostOfRevenue",
        alias = "us-gaap:CostOfGoodsAndServicesSold"
    )]
    pub cost_of_revenue: Option<f64>,

    /// Gross profit. Many filers do not report the subtotal.
    #[xbrl(concept = "us-gaap:GrossProfit")]
    pub gross_profit: Option<f64>,

    /// Research and development expense.
    #[xbrl(concept = "us-gaap:ResearchAndDevelopmentExpense")]
    pub research_and_development: Option<f64>,

    /// Selling, general and administrative expense.
    #[xbrl(concept = "us-gaap:SellingGeneralAndAdministrativeExpense")]
    pub selling_general_and_administrative: Option<f64>,

    /// Total operating expenses. For a filer that reports costs and expenses
    /// as one total, cost of revenue is in it.
    #[xbrl(
        concept = "us-gaap:OperatingExpenses",
        alias = "us-gaap:CostsAndExpenses"
    )]
    pub operating_expenses: Option<f64>,

    /// Operating income or loss.
    #[xbrl(concept = "us-gaap:OperatingIncomeLoss")]
    pub operating_income: Option<f64>,

    /// Interest expense.
    #[xbrl(
        concept = "us-gaap:InterestExpense",
        alias = "us-gaap:InterestExpenseNonoperating",
        alias = "us-gaap:InterestExpenseDebt"
    )]
    pub interest_expense: Option<f64>,

    /// Income or loss before income taxes.
    #[xbrl(
        concept = "us-gaap:IncomeLossFromContinuingOperationsBeforeIncomeTaxesExtraordinaryItemsNoncontrollingInterest",
        alias = "us-gaap:IncomeLossFromContinuingOperationsBeforeIncomeTaxesMinorityInterestAndIncomeLossFromEquityMethodInvestments"
    )]
    pub income_before_tax: Option<f64>,

    /// Income tax expense, or benefit as a negative.
    #[xbrl(concept = "us-gaap:IncomeTaxExpenseBenefit")]
    pub income_tax: Option<f64>,

    /// Net income or loss attributable to the parent's shareholders.
    #[xbrl(concept = "us-gaap:NetIncomeLoss", alias = "us-gaap:ProfitLoss")]
    pub net_income: Option<f64>,

    /// Basic earnings per share.
    #[xbrl(concept = "us-gaap:EarningsPerShareBasic")]
    pub earnings_per_share_basic: Option<f64>,

    /// Diluted earnings per share.
    #[xbrl(concept = "us-gaap:EarningsPerShareDiluted")]
    pub earnings_per_share_diluted: Option<f64>,

    /// Weighted average shares outstanding, basic.
    #[xbrl(concept = "us-gaap:WeightedAverageNumberOfSharesOutstandingBasic")]
    pub weighted_average_shares_basic: Option<f64>,

    /// Weighted average shares outstanding, diluted.
    #[xbrl(concept = "us-gaap:WeightedAverageNumberOfDilutedSharesOutstanding")]
    pub weighted_average_shares_diluted: Option<f64>,
}

/// A cash flow statement: where cash came from and went over one period.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, FromXbrl)]
#[serde(default)]
#[xbrl(duration)]
pub struct CashFlowStatement {
    /// First day of the period these figures cover.
    ///
    /// Set, with `period_end`, on each of
    /// [`Financials::cash_flow_statements`]. `None` on
    /// [`Financials::cash_flow_statement`].
    #[xbrl(period_start)]
    pub period_start: Option<String>,

    /// Last day of the period these figures cover.
    #[xbrl(period_end)]
    pub period_end: Option<String>,

    /// Net cash provided by, or used in, operating activities.
    #[xbrl(
        concept = "us-gaap:NetCashProvidedByUsedInOperatingActivities",
        alias = "us-gaap:NetCashProvidedByUsedInOperatingActivitiesContinuingOperations"
    )]
    pub operating_activities: Option<f64>,

    /// Net cash provided by, or used in, investing activities.
    #[xbrl(
        concept = "us-gaap:NetCashProvidedByUsedInInvestingActivities",
        alias = "us-gaap:NetCashProvidedByUsedInInvestingActivitiesContinuingOperations"
    )]
    pub investing_activities: Option<f64>,

    /// Net cash provided by, or used in, financing activities.
    #[xbrl(
        concept = "us-gaap:NetCashProvidedByUsedInFinancingActivities",
        alias = "us-gaap:NetCashProvidedByUsedInFinancingActivitiesContinuingOperations"
    )]
    pub financing_activities: Option<f64>,

    /// Depreciation and amortization added back to net income.
    #[xbrl(
        concept = "us-gaap:DepreciationDepletionAndAmortization",
        alias = "us-gaap:DepreciationAndAmortization",
        alias = "us-gaap:DepreciationAmortizationAndAccretionNet"
    )]
    pub depreciation_and_amortization: Option<f64>,

    /// Share-based compensation added back to net income.
    #[xbrl(concept = "us-gaap:ShareBasedCompensation")]
    pub share_based_compensation: Option<f64>,

    /// Cash paid for property, plant and equipment, as a positive number.
    #[xbrl(
        concept = "us-gaap:PaymentsToAcquirePropertyPlantAndEquipment",
        alias = "us-gaap:PaymentsToAcquireProductiveAssets"
    )]
    pub capital_expenditures: Option<f64>,

    /// Cash paid in dividends, as a positive number.
    #[xbrl(
        concept = "us-gaap:PaymentsOfDividends",
        alias = "us-gaap:PaymentsOfDividendsCommonStock"
    )]
    pub dividends_paid: Option<f64>,

    /// Cash paid to repurchase the entity's own shares, as a positive number.
    #[xbrl(concept = "us-gaap:PaymentsForRepurchaseOfCommonStock")]
    pub share_repurchases: Option<f64>,

    /// Net change in cash, cash equivalents and restricted cash.
    #[xbrl(
        concept = "us-gaap:CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalentsPeriodIncreaseDecreaseIncludingExchangeRateEffect",
        alias = "us-gaap:CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalentsPeriodIncreaseDecreaseExcludingExchangeRateEffect",
        alias = "us-gaap:CashAndCashEquivalentsPeriodIncreaseDecrease"
    )]
    pub net_change_in_cash: Option<f64>,
}

/// The three statements, each in two views.
///
/// `balance_sheet`, `income_statement` and `cash_flow_statement` take, field
/// by field, the fact that best matches the period the filing reports on — in
/// a 10-Q that is the year to date — and fall back to a comparative or a
/// dimensional breakdown when that is all the filing tags.
///
/// `balance_sheets`, `income_statements` and `cash_flow_statements` hold one
/// statement per period the filing reports, each read from that period
/// alone: the quarter beside the year to date, this year beside last.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, FromXbrl)]
#[serde(default)]
pub struct Financials {
    /// The balance sheet for the reporting date.
    #[xbrl(nested)]
    pub balance_sheet: BalanceSheet,

    /// The income statement for the reporting period.
    #[xbrl(nested)]
    pub income_statement: IncomeStatement,

    /// The cash flow statement for the reporting period.
    #[xbrl(nested)]
    pub cash_flow_statement: CashFlowStatement,

    /// The balance sheet at each date the filing reports one for, latest first.
    #[xbrl(each_period)]
    pub balance_sheets: Vec<BalanceSheet>,

    /// The income statement for each period the filing reports, latest first.
    #[xbrl(each_period)]
    pub income_statements: Vec<IncomeStatement>,

    /// The cash flow statement for each period the filing reports, latest first.
    #[xbrl(each_period)]
    pub cash_flow_statements: Vec<CashFlowStatement>,
}

/// Reads the three statements from a document.
///
/// Fails on the first value that does not convert to its field's type; use
/// [`Document::extract_lenient`] to keep the rest.
pub fn extract_financials(document: &Document) -> Result<Financials> {
    document.extract()
}
