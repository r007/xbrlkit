//! # US-GAAP Taxonomy Extractor
//!
//! Provides high-level functions and structs for extracting standard financial data
//! from XBRL documents based on the US-GAAP taxonomy.
//!
//! This module handles the comprehensive US-GAAP (Generally Accepted Accounting Principles)
//! taxonomy as defined by the FASB. It extracts financial statement data including
//! balance sheets, income statements, cash flow statements, and narrative disclosures.
//! The implementation is optimized for SPAC (Special Purpose Acquisition Company) filings
//! but works with any US-GAAP compliant XBRL document.

use crate::error::Result;
use crate::serde_xbrl::{XbrlDataContext, from_data};
use serde::{Deserialize, Serialize};

/// Represents key data points from the Balance Sheet.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BalanceSheet {
    // --- Assets Section ---
    /// Total assets of the entity.
    #[serde(rename(deserialize = "us-gaap:Assets"), default)]
    pub assets: Option<f64>,

    /// Current assets expected to be realized within one year.
    #[serde(rename(deserialize = "us-gaap:AssetsCurrent"), default)]
    pub assets_current: Option<f64>,

    /// Assets held in trust (common in SPACs).
    #[serde(rename(deserialize = "us-gaap:AssetsHeldInTrust"), default)]
    pub assets_held_in_trust: Option<f64>,

    /// Non-current assets held in trust.
    #[serde(rename(deserialize = "us-gaap:AssetsHeldInTrustNoncurrent"), default)]
    pub assets_held_in_trust_noncurrent: Option<f64>,

    /// Cash and cash equivalents at carrying value.
    #[serde(
        rename(deserialize = "us-gaap:CashAndCashEquivalentsAtCarryingValue"),
        default
    )]
    pub cash_and_cash_equivalents: Option<f64>,

    /// Total of cash, cash equivalents, restricted cash, and restricted cash equivalents.
    #[serde(
        rename(
            deserialize = "us-gaap:CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalents"
        ),
        default
    )]
    pub cash_cash_equivalents_restricted_cash: Option<f64>,

    /// Prepaid expenses expected to be consumed within one year.
    #[serde(rename(deserialize = "us-gaap:PrepaidExpenseCurrent"), default)]
    pub prepaid_expense_current: Option<f64>,

    /// Costs deferred in connection with public or private offerings.
    #[serde(rename(deserialize = "us-gaap:DeferredOfferingCosts"), default)]
    pub deferred_offering_costs: Option<f64>,

    /// Assets held as deposits.
    #[serde(rename(deserialize = "us-gaap:DepositAssets"), default)]
    pub deposit_assets: Option<f64>,

    // --- Liabilities Section ---
    /// Total liabilities of the entity.
    #[serde(rename(deserialize = "us-gaap:Liabilities"), default)]
    pub liabilities: Option<f64>,

    /// Current liabilities expected to be settled within one year.
    #[serde(rename(deserialize = "us-gaap:LiabilitiesCurrent"), default)]
    pub liabilities_current: Option<f64>,

    /// Amounts owed to trade creditors within one year.
    #[serde(rename(deserialize = "us-gaap:AccountsPayableCurrent"), default)]
    pub accounts_payable_current: Option<f64>,

    /// Accrued liabilities due within one year.
    #[serde(rename(deserialize = "us-gaap:AccruedLiabilitiesCurrent"), default)]
    pub accrued_liabilities_current: Option<f64>,

    /// Other current liabilities not elsewhere classified.
    #[serde(rename(deserialize = "us-gaap:OtherLiabilitiesCurrent"), default)]
    pub other_liabilities_current: Option<f64>,

    /// Other notes payable due within one year.
    #[serde(rename(deserialize = "us-gaap:OtherNotesPayableCurrent"), default)]
    pub other_notes_payable_current: Option<f64>,

    /// Deferred compensation liability classified as non-current.
    #[serde(
        rename(deserialize = "us-gaap:DeferredCompensationLiabilityClassifiedNoncurrent"),
        default
    )]
    pub deferred_compensation_liability_noncurrent: Option<f64>,

    // --- Equity Section ---
    /// Total stockholders' equity.
    #[serde(rename(deserialize = "us-gaap:StockholdersEquity"), default)]
    pub stockholders_equity: Option<f64>,

    /// Par or stated value of common stock issued.
    #[serde(rename(deserialize = "us-gaap:CommonStockValue"), default)]
    pub common_stock_value: Option<f64>,

    /// Additional paid-in capital from stock issuances.
    #[serde(rename(deserialize = "us-gaap:AdditionalPaidInCapital"), default)]
    pub additional_paid_in_capital: Option<f64>,

    /// Retained earnings or accumulated deficit.
    #[serde(
        rename(deserialize = "us-gaap:RetainedEarningsAccumulatedDeficit"),
        default
    )]
    pub retained_earnings_accumulated_deficit: Option<f64>,

    /// Accumulated other comprehensive income or loss, net of tax.
    #[serde(
        rename(deserialize = "us-gaap:AccumulatedOtherComprehensiveIncomeLossNetOfTax"),
        default
    )]
    pub accumulated_other_comprehensive_income_loss: Option<f64>,

    /// Total liabilities and stockholders' equity (should equal total assets).
    #[serde(
        rename(deserialize = "us-gaap:LiabilitiesAndStockholdersEquity"),
        default
    )]
    pub liabilities_and_stockholders_equity: Option<f64>,
}

/// Represents key data points from the Income Statement.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IncomeStatement {
    /// Total revenues recognized during the period.
    #[serde(rename(deserialize = "us-gaap:Revenues"), default)]
    pub revenues: Option<f64>,

    /// Total operating expenses incurred during the period.
    #[serde(rename(deserialize = "us-gaap:OperatingExpenses"), default)]
    pub operating_expenses: Option<f64>,

    /// Total operating costs and expenses.
    #[serde(rename(deserialize = "us-gaap:OperatingCostsAndExpenses"), default)]
    pub operating_costs_and_expenses: Option<f64>,

    /// General and administrative expenses.
    #[serde(
        rename(deserialize = "us-gaap:GeneralAndAdministrativeExpense"),
        default
    )]
    pub general_and_administrative_expense: Option<f64>,

    /// Utilities operating expenses for products and services.
    #[serde(
        rename(deserialize = "us-gaap:UtilitiesOperatingExpenseProductsAndServices"),
        default
    )]
    pub utilities_operating_expense: Option<f64>,

    /// Non-operating income and expenses.
    #[serde(rename(deserialize = "us-gaap:NonoperatingIncomeExpense"), default)]
    pub nonoperating_income_expense: Option<f64>,

    /// Interest and other income.
    #[serde(rename(deserialize = "us-gaap:InterestAndOtherIncome"), default)]
    pub interest_and_other_income: Option<f64>,

    /// Other interest income.
    #[serde(rename(deserialize = "us-gaap:InterestIncomeOther"), default)]
    pub interest_income_other: Option<f64>,

    /// Investment income from interest.
    #[serde(rename(deserialize = "us-gaap:InvestmentIncomeInterest"), default)]
    pub investment_income_interest: Option<f64>,

    /// Investment income from dividends.
    #[serde(rename(deserialize = "us-gaap:InvestmentIncomeDividend"), default)]
    pub investment_income_dividend: Option<f64>,

    /// Other underwriting expenses.
    #[serde(rename(deserialize = "us-gaap:OtherUnderwritingExpense"), default)]
    pub other_underwriting_expense: Option<f64>,

    /// Payments for fees.
    #[serde(rename(deserialize = "us-gaap:PaymentsForFees"), default)]
    pub payments_for_fees: Option<f64>,

    /// Net income or loss for the period.
    #[serde(rename(deserialize = "us-gaap:NetIncomeLoss"), default)]
    pub net_income_loss: Option<f64>,

    /// Net income or loss available to common stockholders (basic).
    #[serde(
        rename(deserialize = "us-gaap:NetIncomeLossAvailableToCommonStockholdersBasic"),
        default
    )]
    pub net_income_loss_available_to_common_stockholders_basic: Option<f64>,

    /// Comprehensive income, net of tax.
    #[serde(rename(deserialize = "us-gaap:ComprehensiveIncomeNetOfTax"), default)]
    pub comprehensive_income_net_of_tax: Option<f64>,

    /// Basic earnings per share.
    #[serde(rename(deserialize = "us-gaap:EarningsPerShareBasic"), default)]
    pub earnings_per_share_basic: Option<f64>,

    /// Diluted earnings per share.
    #[serde(rename(deserialize = "us-gaap:EarningsPerShareDiluted"), default)]
    pub earnings_per_share_diluted: Option<f64>,

    /// Weighted average number of shares outstanding (basic).
    #[serde(
        rename(deserialize = "us-gaap:WeightedAverageNumberOfSharesOutstandingBasic"),
        default
    )]
    pub weighted_average_shares_outstanding_basic: Option<f64>,

    /// Weighted average number of diluted shares outstanding.
    #[serde(
        rename(deserialize = "us-gaap:WeightedAverageNumberOfDilutedSharesOutstanding"),
        default
    )]
    pub weighted_average_shares_outstanding_diluted: Option<f64>,
}

/// Represents key data points from the Statement of Cash Flows.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CashFlowStatement {
    /// Net cash provided by or used in operating activities.
    #[serde(
        rename(deserialize = "us-gaap:NetCashProvidedByUsedInOperatingActivities"),
        default
    )]
    pub net_cash_provided_by_operating_activities: Option<f64>,

    /// Net cash provided by or used in investing activities.
    #[serde(
        rename(deserialize = "us-gaap:NetCashProvidedByUsedInInvestingActivities"),
        default
    )]
    pub net_cash_provided_by_investing_activities: Option<f64>,

    /// Net cash provided by or used in financing activities.
    #[serde(
        rename(deserialize = "us-gaap:NetCashProvidedByUsedInFinancingActivities"),
        default
    )]
    pub net_cash_provided_by_financing_activities: Option<f64>,

    /// Net increase or decrease in cash and cash equivalents during the period.
    #[serde(
        rename(
            deserialize = "us-gaap:CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalentsPeriodIncreaseDecreaseExcludingExchangeRateEffect"
        ),
        default
    )]
    pub cash_and_cash_equivalents_period_increase_decrease: Option<f64>,

    /// Proceeds from initial public offering.
    #[serde(
        rename(deserialize = "us-gaap:ProceedsFromIssuanceInitialPublicOffering"),
        default
    )]
    pub proceeds_from_issuance_initial_public_offering: Option<f64>,

    /// Proceeds from private placement issuance.
    #[serde(
        rename(deserialize = "us-gaap:ProceedsFromIssuanceOfPrivatePlacement"),
        default
    )]
    pub proceeds_from_issuance_of_private_placement: Option<f64>,

    /// Proceeds from notes payable.
    #[serde(rename(deserialize = "us-gaap:ProceedsFromNotesPayable"), default)]
    pub proceeds_from_notes_payable: Option<f64>,

    /// Proceeds from related party debt.
    #[serde(rename(deserialize = "us-gaap:ProceedsFromRelatedPartyDebt"), default)]
    pub proceeds_from_related_party_debt: Option<f64>,

    /// Payments to acquire investments.
    #[serde(rename(deserialize = "us-gaap:PaymentsToAcquireInvestments"), default)]
    pub payments_to_acquire_investments: Option<f64>,

    /// Payments of debt issuance costs.
    #[serde(rename(deserialize = "us-gaap:PaymentsOfDebtIssuanceCosts"), default)]
    pub payments_of_debt_issuance_costs: Option<f64>,

    /// Repayments of related party debt.
    #[serde(rename(deserialize = "us-gaap:RepaymentsOfRelatedPartyDebt"), default)]
    pub repayments_of_related_party_debt: Option<f64>,

    /// Increase or decrease in accrued liabilities.
    #[serde(
        rename(deserialize = "us-gaap:IncreaseDecreaseInAccruedLiabilities"),
        default
    )]
    pub increase_decrease_in_accrued_liabilities: Option<f64>,

    /// Increase or decrease in prepaid expenses.
    #[serde(
        rename(deserialize = "us-gaap:IncreaseDecreaseInPrepaidExpense"),
        default
    )]
    pub increase_decrease_in_prepaid_expense: Option<f64>,
}

/// Represents detailed information about stock issuances and equity transactions.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EquityDetails {
    /// Number of common stock shares authorized.
    #[serde(rename(deserialize = "us-gaap:CommonStockSharesAuthorized"), default)]
    pub common_stock_shares_authorized: Option<f64>,

    /// Number of common stock shares issued.
    #[serde(rename(deserialize = "us-gaap:CommonStockSharesIssued"), default)]
    pub common_stock_shares_issued: Option<f64>,

    /// Number of common stock shares outstanding.
    #[serde(rename(deserialize = "us-gaap:CommonStockSharesOutstanding"), default)]
    pub common_stock_shares_outstanding: Option<f64>,

    /// Par or stated value per common stock share.
    #[serde(
        rename(deserialize = "us-gaap:CommonStockParOrStatedValuePerShare"),
        default
    )]
    pub common_stock_par_value_per_share: Option<f64>,

    /// Voting rights description for common stock.
    #[serde(rename(deserialize = "us-gaap:CommonStockVotingRights"), default)]
    pub common_stock_voting_rights: Option<String>,

    /// Total shares outstanding.
    #[serde(rename(deserialize = "us-gaap:SharesOutstanding"), default)]
    pub shares_outstanding: Option<f64>,

    /// Price per share for shares issued.
    #[serde(rename(deserialize = "us-gaap:SharesIssuedPricePerShare"), default)]
    pub shares_issued_price_per_share: Option<f64>,

    /// Current share price.
    #[serde(rename(deserialize = "us-gaap:SharePrice"), default)]
    pub share_price: Option<f64>,

    /// Price per share for stock sales.
    #[serde(rename(deserialize = "us-gaap:SaleOfStockPricePerShare"), default)]
    pub sale_of_stock_price_per_share: Option<f64>,

    /// Number of shares issued in stock sale transaction.
    #[serde(
        rename(deserialize = "us-gaap:SaleOfStockNumberOfSharesIssuedInTransaction"),
        default
    )]
    pub sale_of_stock_number_of_shares_issued: Option<f64>,

    /// Shares issued during period - new issues.
    #[serde(
        rename(deserialize = "us-gaap:StockIssuedDuringPeriodSharesNewIssues"),
        default
    )]
    pub stock_issued_during_period_shares_new_issues: Option<f64>,

    /// Shares issued during period for share-based compensation.
    #[serde(
        rename(deserialize = "us-gaap:StockIssuedDuringPeriodSharesShareBasedCompensation"),
        default
    )]
    pub stock_issued_during_period_shares_share_based_compensation: Option<f64>,

    /// Other shares issued during period.
    #[serde(
        rename(deserialize = "us-gaap:StockIssuedDuringPeriodSharesOther"),
        default
    )]
    pub stock_issued_during_period_shares_other: Option<f64>,

    /// Value of new stock issues during period.
    #[serde(
        rename(deserialize = "us-gaap:StockIssuedDuringPeriodValueNewIssues"),
        default
    )]
    pub stock_issued_during_period_value_new_issues: Option<f64>,

    /// Value of other stock issued during period.
    #[serde(
        rename(deserialize = "us-gaap:StockIssuedDuringPeriodValueOther"),
        default
    )]
    pub stock_issued_during_period_value_other: Option<f64>,

    /// General stock issued amount.
    #[serde(rename(deserialize = "us-gaap:StockIssued1"), default)]
    pub stock_issued: Option<f64>,
}

/// Represents information about temporary equity and warrants (common in SPACs).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemporaryEquityAndWarrants {
    /// Carrying amount of temporary equity attributable to parent.
    #[serde(
        rename(deserialize = "us-gaap:TemporaryEquityCarryingAmountAttributableToParent"),
        default
    )]
    pub temporary_equity_carrying_amount: Option<f64>,

    /// Accretion of temporary equity to redemption value.
    #[serde(
        rename(deserialize = "us-gaap:TemporaryEquityAccretionToRedemptionValue"),
        default
    )]
    pub temporary_equity_accretion_to_redemption_value: Option<f64>,

    /// Adjustment to temporary equity accretion to redemption value.
    #[serde(
        rename(deserialize = "us-gaap:TemporaryEquityAccretionToRedemptionValueAdjustment"),
        default
    )]
    pub temporary_equity_accretion_adjustment: Option<f64>,

    /// Number of temporary equity shares issued.
    #[serde(rename(deserialize = "us-gaap:TemporaryEquitySharesIssued"), default)]
    pub temporary_equity_shares_issued: Option<f64>,

    /// Number of temporary equity shares outstanding.
    #[serde(
        rename(deserialize = "us-gaap:TemporaryEquitySharesOutstanding"),
        default
    )]
    pub temporary_equity_shares_outstanding: Option<f64>,

    /// Exercise price of warrants or rights.
    #[serde(
        rename(deserialize = "us-gaap:ClassOfWarrantOrRightExercisePriceOfWarrantsOrRights1"),
        default
    )]
    pub warrant_exercise_price: Option<f64>,

    /// Number of securities called by warrants or rights.
    #[serde(
        rename(
            deserialize = "us-gaap:ClassOfWarrantOrRightNumberOfSecuritiesCalledByWarrantsOrRights"
        ),
        default
    )]
    pub warrant_number_of_securities_called: Option<f64>,

    /// Terms of outstanding warrants and rights.
    #[serde(
        rename(deserialize = "us-gaap:WarrantsAndRightsOutstandingTerm"),
        default
    )]
    pub warrants_and_rights_outstanding_term: Option<String>,
}

/// Represents acquisition and business combination information (relevant for SPACs).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BusinessCombinations {
    /// Consideration transferred in asset acquisition.
    #[serde(
        rename(deserialize = "us-gaap:AssetAcquisitionConsiderationTransferred"),
        default
    )]
    pub asset_acquisition_consideration_transferred: Option<f64>,

    /// Percentage of voting interests acquired in business acquisition.
    #[serde(
        rename(deserialize = "us-gaap:BusinessAcquisitionPercentageOfVotingInterestsAcquired"),
        default
    )]
    pub business_acquisition_percentage_of_voting_interests: Option<f64>,
}

/// Represents comprehensive income components.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComprehensiveIncomeDetails {
    /// Other comprehensive income from unrealized gains/losses on securities.
    #[serde(
        rename(
            deserialize = "us-gaap:OtherComprehensiveIncomeUnrealizedHoldingGainLossOnSecuritiesArisingDuringPeriodBeforeTax"
        ),
        default
    )]
    pub other_comprehensive_income_unrealized_securities: Option<f64>,

    /// Other comprehensive income from foreign currency transactions and translations.
    #[serde(
        rename(
            deserialize = "us-gaap:OtherComprehensiveIncomeLossForeignCurrencyTransactionAndTranslationReclassificationAdjustmentFromAOCIRealizedUponSaleOrLiquidationBeforeTax"
        ),
        default
    )]
    pub other_comprehensive_income_foreign_currency: Option<f64>,

    /// Other comprehensive income reclassification for held-to-maturity transfers.
    #[serde(
        rename(
            deserialize = "us-gaap:OtherComprehensiveIncomeReclassificationAdjustmentForHeldToMaturityTransferredToAvailableForSaleSecuritiesBeforeTax"
        ),
        default
    )]
    pub other_comprehensive_income_htm_reclassification: Option<f64>,

    /// Adjustments to additional paid-in capital for stock issuance costs.
    #[serde(
        rename(
            deserialize = "us-gaap:AdjustmentsToAdditionalPaidInCapitalStockIssuedIssuanceCosts"
        ),
        default
    )]
    pub adjustments_to_paid_in_capital_issuance_costs: Option<f64>,
}

/// Represents key narrative text blocks for LLM analysis.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Narratives {
    /// Description of the nature of the entity's operations.
    #[serde(rename(deserialize = "us-gaap:NatureOfOperations"), default)]
    pub nature_of_operations: Option<String>,

    /// Disclosure text block for commitments and contingencies.
    #[serde(
        rename(deserialize = "us-gaap:CommitmentsAndContingenciesDisclosureTextBlock"),
        default
    )]
    pub commitments_and_contingencies: Option<String>,

    /// Text block describing significant accounting policies.
    #[serde(
        rename(deserialize = "us-gaap:SignificantAccountingPoliciesTextBlock"),
        default
    )]
    pub significant_accounting_policies: Option<String>,

    /// Text block describing consolidation policy.
    #[serde(rename(deserialize = "us-gaap:ConsolidationPolicyTextBlock"), default)]
    pub consolidation_policy: Option<String>,

    /// Text block describing subsequent events.
    #[serde(rename(deserialize = "us-gaap:SubsequentEventsTextBlock"), default)]
    pub subsequent_events: Option<String>,

    /// Text block describing cash and cash equivalents policy.
    #[serde(
        rename(deserialize = "us-gaap:CashAndCashEquivalentsPolicyTextBlock"),
        default
    )]
    pub cash_and_cash_equivalents_policy: Option<String>,

    /// Text block describing earnings per share policy.
    #[serde(
        rename(deserialize = "us-gaap:EarningsPerSharePolicyTextBlock"),
        default
    )]
    pub earnings_per_share_policy: Option<String>,

    /// Policy for fair value of financial instruments.
    #[serde(
        rename(deserialize = "us-gaap:FairValueOfFinancialInstrumentsPolicy"),
        default
    )]
    pub fair_value_of_financial_instruments_policy: Option<String>,

    /// Text block describing income tax policy.
    #[serde(rename(deserialize = "us-gaap:IncomeTaxPolicyTextBlock"), default)]
    pub income_tax_policy: Option<String>,

    /// Text block describing new accounting pronouncements policy.
    #[serde(
        rename(deserialize = "us-gaap:NewAccountingPronouncementsPolicyPolicyTextBlock"),
        default
    )]
    pub new_accounting_pronouncements_policy: Option<String>,

    /// Text block disclosing related party transactions.
    #[serde(
        rename(deserialize = "us-gaap:RelatedPartyTransactionsDisclosureTextBlock"),
        default
    )]
    pub related_party_transactions: Option<String>,

    /// Text block for stockholders' equity note disclosure.
    #[serde(
        rename(deserialize = "us-gaap:StockholdersEquityNoteDisclosureTextBlock"),
        default
    )]
    pub stockholders_equity_note: Option<String>,

    /// Text block for fair value assets measured on recurring basis.
    #[serde(
        rename(deserialize = "us-gaap:FairValueAssetsMeasuredOnRecurringBasisTextBlock"),
        default
    )]
    pub fair_value_assets_measured_recurring_basis: Option<String>,

    /// Text block for schedule of earnings per share basic and diluted.
    #[serde(
        rename(deserialize = "us-gaap:ScheduleOfEarningsPerShareBasicAndDilutedTableTextBlock"),
        default
    )]
    pub schedule_of_earnings_per_share: Option<String>,

    /// Text block for segment reporting disclosure.
    #[serde(
        rename(deserialize = "us-gaap:SegmentReportingDisclosureTextBlock"),
        default
    )]
    pub segment_reporting: Option<String>,

    /// Text block describing subsidiary of limited liability company or limited partnership.
    #[serde(
        rename(
            deserialize = "us-gaap:ScheduleOfSubsidiaryOfLimitedLiabilityCompanyOrLimitedPartnershipDescriptionTextBlock"
        ),
        default
    )]
    pub subsidiary_description: Option<String>,

    /// Text block for shares subject to mandatory redemption policy.
    #[serde(
        rename(
            deserialize = "us-gaap:SharesSubjectToMandatoryRedemptionChangesInRedemptionValuePolicyTextBlock"
        ),
        default
    )]
    pub shares_subject_to_mandatory_redemption_policy: Option<String>,
}

/// A composite structure holding all extracted US-GAAP financial data.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Financials {
    /// Balance sheet information including assets, liabilities, and equity.
    pub balance_sheet: BalanceSheet,

    /// Income statement information including revenues, expenses, and earnings.
    pub income_statement: IncomeStatement,

    /// Cash flow statement information.
    pub cash_flow_statement: CashFlowStatement,

    /// Detailed equity and stock information.
    pub equity_details: EquityDetails,

    /// Temporary equity and warrant information (SPAC-specific).
    pub temporary_equity_and_warrants: TemporaryEquityAndWarrants,

    /// Business combination and acquisition information.
    pub business_combinations: BusinessCombinations,

    /// Comprehensive income details.
    pub comprehensive_income_details: ComprehensiveIncomeDetails,

    /// Narrative disclosures for LLM analysis.
    pub narratives: Narratives,
}

/// Extracts a comprehensive set of financial data from an XBRL document.
///
/// This function uses the high-performance `serde`-based deserializer to map
/// XBRL concepts directly to the `Financials` struct.
pub fn extract_financials(context: &XbrlDataContext) -> Result<Financials> {
    from_data(context)
}
