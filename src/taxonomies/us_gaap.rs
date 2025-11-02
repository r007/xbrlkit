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
    #[serde(rename = "us-gaap:Assets", default)]
    pub assets: Option<f64>,

    /// Current assets expected to be realized within one year.
    #[serde(rename = "us-gaap:AssetsCurrent", default)]
    pub assets_current: Option<f64>,

    /// Assets held in trust (common in SPACs).
    #[serde(rename = "us-gaap:AssetsHeldInTrust", default)]
    pub assets_held_in_trust: Option<f64>,

    /// Non-current assets held in trust.
    #[serde(rename = "us-gaap:AssetsHeldInTrustNoncurrent", default)]
    pub assets_held_in_trust_noncurrent: Option<f64>,

    /// Cash (not including cash equivalents).
    #[serde(rename = "us-gaap:Cash", default)]
    pub cash: Option<f64>,

    /// Cash and cash equivalents at carrying value.
    #[serde(rename = "us-gaap:CashAndCashEquivalentsAtCarryingValue", default)]
    pub cash_and_cash_equivalents: Option<f64>,

    /// Cash equivalents at carrying value.
    #[serde(rename = "us-gaap:CashEquivalentsAtCarryingValue", default)]
    pub cash_equivalents_at_carrying_value: Option<f64>,

    /// Total of cash, cash equivalents, restricted cash, and restricted cash equivalents.
    #[serde(
        rename = "us-gaap:CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalents",
        default
    )]
    pub cash_cash_equivalents_restricted_cash: Option<f64>,

    /// Marketable securities classified as noncurrent.
    #[serde(rename = "us-gaap:MarketableSecuritiesNoncurrent", default)]
    pub marketable_securities_noncurrent: Option<f64>,

    /// Prepaid expenses expected to be consumed within one year.
    #[serde(rename = "us-gaap:PrepaidExpenseCurrent", default)]
    pub prepaid_expense_current: Option<f64>,

    /// Prepaid expenses expected to be consumed beyond one year.
    #[serde(rename = "us-gaap:PrepaidExpenseNoncurrent", default)]
    pub prepaid_expense_noncurrent: Option<f64>,

    /// Prepaid insurance premiums.
    #[serde(rename = "us-gaap:PrepaidInsurance", default)]
    pub prepaid_insurance: Option<f64>,

    /// Other prepaid expenses not elsewhere classified.
    #[serde(rename = "us-gaap:OtherPrepaidExpenseCurrent", default)]
    pub other_prepaid_expense_current: Option<f64>,

    /// Costs deferred in connection with public or private offerings.
    #[serde(rename = "us-gaap:DeferredOfferingCosts", default)]
    pub deferred_offering_costs: Option<f64>,

    /// Deferred costs not elsewhere classified.
    #[serde(rename = "us-gaap:DeferredCosts", default)]
    pub deferred_costs: Option<f64>,

    /// Assets held as deposits.
    #[serde(rename = "us-gaap:DepositAssets", default)]
    pub deposit_assets: Option<f64>,

    /// Other receivables not elsewhere classified.
    #[serde(rename = "us-gaap:OtherReceivables", default)]
    pub other_receivables: Option<f64>,

    /// Other prepaid expenses classified as noncurrent.
    #[serde(rename = "us-gaap:PrepaidExpenseOtherNoncurrent", default)]
    pub prepaid_expense_other_noncurrent: Option<f64>,

    // --- Liabilities Section ---
    /// Total liabilities of the entity.
    #[serde(rename = "us-gaap:Liabilities", default)]
    pub liabilities: Option<f64>,

    /// Current liabilities expected to be settled within one year.
    #[serde(rename = "us-gaap:LiabilitiesCurrent", default)]
    pub liabilities_current: Option<f64>,

    /// Amounts owed to trade creditors within one year.
    #[serde(rename = "us-gaap:AccountsPayableCurrent", default)]
    pub accounts_payable_current: Option<f64>,

    /// Accrued liabilities due within one year.
    #[serde(rename = "us-gaap:AccruedLiabilitiesCurrent", default)]
    pub accrued_liabilities_current: Option<f64>,

    /// Other current liabilities not elsewhere classified.
    #[serde(rename = "us-gaap:OtherLiabilitiesCurrent", default)]
    pub other_liabilities_current: Option<f64>,

    /// Other liabilities not elsewhere classified (non-current).
    #[serde(rename = "us-gaap:OtherLiabilities", default)]
    pub other_liabilities: Option<f64>,

    /// Notes payable.
    #[serde(rename = "us-gaap:NotesPayable", default)]
    pub notes_payable: Option<f64>,

    /// Notes payable due within one year.
    #[serde(rename = "us-gaap:NotesPayableCurrent", default)]
    pub notes_payable_current: Option<f64>,

    /// Long-term notes payable.
    #[serde(rename = "us-gaap:LongTermNotesPayable", default)]
    pub long_term_notes_payable: Option<f64>,

    /// Loans payable.
    #[serde(rename = "us-gaap:LoansPayable", default)]
    pub loans_payable: Option<f64>,

    /// Short-term borrowings.
    #[serde(rename = "us-gaap:ShortTermBorrowings", default)]
    pub short_term_borrowings: Option<f64>,

    /// Unsecured debt.
    #[serde(rename = "us-gaap:UnsecuredDebt", default)]
    pub unsecured_debt: Option<f64>,

    /// Bank overdrafts.
    #[serde(rename = "us-gaap:BankOverdrafts", default)]
    pub bank_overdrafts: Option<f64>,

    /// Other notes payable due within one year.
    #[serde(rename = "us-gaap:OtherNotesPayableCurrent", default)]
    pub other_notes_payable_current: Option<f64>,

    /// Other notes payable (general, not classified by maturity).
    #[serde(rename = "us-gaap:OtherNotesPayable", default)]
    pub other_notes_payable: Option<f64>,

    /// Interest payable, current and noncurrent.
    #[serde(rename = "us-gaap:InterestPayableCurrentAndNoncurrent", default)]
    pub interest_payable_current_and_noncurrent: Option<f64>,

    /// Long-term debt.
    #[serde(rename = "us-gaap:LongTermDebt", default)]
    pub long_term_debt: Option<f64>,

    /// Deferred compensation liability classified as non-current.
    #[serde(
        rename = "us-gaap:DeferredCompensationLiabilityClassifiedNoncurrent",
        default
    )]
    pub deferred_compensation_liability_noncurrent: Option<f64>,

    // --- Equity Section ---
    /// Total stockholders' equity.
    #[serde(rename = "us-gaap:StockholdersEquity", default)]
    pub stockholders_equity: Option<f64>,

    /// Par or stated value of common stock issued.
    #[serde(rename = "us-gaap:CommonStockValue", default)]
    pub common_stock_value: Option<f64>,

    /// Preferred stock value.
    #[serde(rename = "us-gaap:PreferredStockValue", default)]
    pub preferred_stock_value: Option<f64>,

    /// Preferred stock par value per share.
    #[serde(rename = "us-gaap:PreferredStockParOrStatedValuePerShare", default)]
    pub preferred_stock_par_value_per_share: Option<f64>,

    /// Number of preferred shares authorized.
    #[serde(rename = "us-gaap:PreferredStockSharesAuthorized", default)]
    pub preferred_stock_shares_authorized: Option<f64>,

    /// Number of preferred shares issued.
    #[serde(rename = "us-gaap:PreferredStockSharesIssued", default)]
    pub preferred_stock_shares_issued: Option<f64>,

    /// Number of preferred shares outstanding.
    #[serde(rename = "us-gaap:PreferredStockSharesOutstanding", default)]
    pub preferred_stock_shares_outstanding: Option<f64>,

    /// Additional paid-in capital from stock issuances.
    #[serde(rename = "us-gaap:AdditionalPaidInCapital", default)]
    pub additional_paid_in_capital: Option<f64>,

    /// Retained earnings or accumulated deficit.
    #[serde(rename = "us-gaap:RetainedEarningsAccumulatedDeficit", default)]
    pub retained_earnings_accumulated_deficit: Option<f64>,

    /// Accumulated other comprehensive income or loss, net of tax.
    #[serde(
        rename = "us-gaap:AccumulatedOtherComprehensiveIncomeLossNetOfTax",
        default
    )]
    pub accumulated_other_comprehensive_income_loss: Option<f64>,

    /// Total liabilities and stockholders' equity (should equal total assets).
    #[serde(rename = "us-gaap:LiabilitiesAndStockholdersEquity", default)]
    pub liabilities_and_stockholders_equity: Option<f64>,
}

/// Represents key data points from the Income Statement.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IncomeStatement {
    /// Total revenues recognized during the period.
    #[serde(rename = "us-gaap:Revenues", default)]
    pub revenues: Option<f64>,

    /// Operating income or loss.
    #[serde(rename = "us-gaap:OperatingIncomeLoss", default)]
    pub operating_income_loss: Option<f64>,

    /// Total operating expenses incurred during the period.
    #[serde(rename = "us-gaap:OperatingExpenses", default)]
    pub operating_expenses: Option<f64>,

    /// Total operating costs and expenses.
    #[serde(rename = "us-gaap:OperatingCostsAndExpenses", default)]
    pub operating_costs_and_expenses: Option<f64>,

    /// General and administrative expenses.
    #[serde(rename = "us-gaap:GeneralAndAdministrativeExpense", default)]
    pub general_and_administrative_expense: Option<f64>,

    /// Administrative fees expense.
    #[serde(rename = "us-gaap:AdministrativeFeesExpense", default)]
    pub administrative_fees_expense: Option<f64>,

    /// Professional and contract services expense.
    #[serde(rename = "us-gaap:ProfessionalAndContractServicesExpense", default)]
    pub professional_and_contract_services_expense: Option<f64>,

    /// Professional fees.
    #[serde(rename = "us-gaap:ProfessionalFees", default)]
    pub professional_fees: Option<f64>,

    /// Travel and entertainment expense.
    #[serde(rename = "us-gaap:TravelAndEntertainmentExpense", default)]
    pub travel_and_entertainment_expense: Option<f64>,

    /// SPAC sponsor fees.
    #[serde(rename = "us-gaap:SponsorFees", default)]
    pub sponsor_fees: Option<f64>,

    /// Income tax expense or benefit.
    #[serde(rename = "us-gaap:IncomeTaxExpenseBenefit", default)]
    pub income_tax_expense_benefit: Option<f64>,

    /// Utilities operating expenses for products and services.
    #[serde(
        rename = "us-gaap:UtilitiesOperatingExpenseProductsAndServices",
        default
    )]
    pub utilities_operating_expense: Option<f64>,

    /// Non-operating income and expenses.
    #[serde(rename = "us-gaap:NonoperatingIncomeExpense", default)]
    pub nonoperating_income_expense: Option<f64>,

    /// Non-operating interest expense.
    #[serde(rename = "us-gaap:InterestExpenseNonoperating", default)]
    pub interest_expense_nonoperating: Option<f64>,

    /// Interest and other income.
    #[serde(rename = "us-gaap:InterestAndOtherIncome", default)]
    pub interest_and_other_income: Option<f64>,

    /// Other interest income.
    #[serde(rename = "us-gaap:InterestIncomeOther", default)]
    pub interest_income_other: Option<f64>,

    /// Investment income from interest.
    #[serde(rename = "us-gaap:InvestmentIncomeInterest", default)]
    pub investment_income_interest: Option<f64>,

    /// Investment income from dividends.
    #[serde(rename = "us-gaap:InvestmentIncomeDividend", default)]
    pub investment_income_dividend: Option<f64>,

    /// Gain or loss on investments.
    #[serde(rename = "us-gaap:GainLossOnInvestments", default)]
    pub gain_loss_on_investments: Option<f64>,

    /// Fair value adjustment of warrants.
    #[serde(rename = "us-gaap:FairValueAdjustmentOfWarrants", default)]
    pub fair_value_adjustment_of_warrants: Option<f64>,

    /// Other underwriting expenses.
    #[serde(rename = "us-gaap:OtherUnderwritingExpense", default)]
    pub other_underwriting_expense: Option<f64>,

    /// Payments for fees.
    #[serde(rename = "us-gaap:PaymentsForFees", default)]
    pub payments_for_fees: Option<f64>,

    /// Net income or loss for the period.
    #[serde(rename = "us-gaap:NetIncomeLoss", default)]
    pub net_income_loss: Option<f64>,

    /// Profit or loss (alternative name for net income/loss).
    #[serde(rename = "us-gaap:ProfitLoss", default)]
    pub profit_loss: Option<f64>,

    /// Net income or loss available to common stockholders (basic).
    #[serde(
        rename = "us-gaap:NetIncomeLossAvailableToCommonStockholdersBasic",
        default
    )]
    pub net_income_loss_available_to_common_stockholders_basic: Option<f64>,

    /// Net income or loss available to common stockholders (diluted).
    #[serde(
        rename = "us-gaap:NetIncomeLossAvailableToCommonStockholdersDiluted",
        default
    )]
    pub net_income_loss_available_to_common_stockholders_diluted: Option<f64>,

    /// Comprehensive income, net of tax.
    #[serde(rename = "us-gaap:ComprehensiveIncomeNetOfTax", default)]
    pub comprehensive_income_net_of_tax: Option<f64>,

    /// Basic earnings per share.
    #[serde(rename = "us-gaap:EarningsPerShareBasic", default)]
    pub earnings_per_share_basic: Option<f64>,

    /// Diluted earnings per share.
    #[serde(rename = "us-gaap:EarningsPerShareDiluted", default)]
    pub earnings_per_share_diluted: Option<f64>,

    /// Weighted average number of shares outstanding (basic).
    #[serde(
        rename = "us-gaap:WeightedAverageNumberOfSharesOutstandingBasic",
        default
    )]
    pub weighted_average_shares_outstanding_basic: Option<f64>,

    /// Weighted average number of diluted shares outstanding.
    #[serde(
        rename = "us-gaap:WeightedAverageNumberOfDilutedSharesOutstanding",
        default
    )]
    pub weighted_average_shares_outstanding_diluted: Option<f64>,

    /// Antidilutive securities excluded from computation of earnings per share.
    #[serde(
        rename = "us-gaap:AntidilutiveSecuritiesExcludedFromComputationOfEarningsPerShareAmount",
        default
    )]
    pub antidilutive_securities_excluded_from_eps: Option<f64>,
}

/// Represents key data points from the Statement of Cash Flows.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CashFlowStatement {
    /// Net cash provided by or used in operating activities.
    #[serde(rename = "us-gaap:NetCashProvidedByUsedInOperatingActivities", default)]
    pub net_cash_provided_by_operating_activities: Option<f64>,

    /// Net cash provided by or used in investing activities.
    #[serde(rename = "us-gaap:NetCashProvidedByUsedInInvestingActivities", default)]
    pub net_cash_provided_by_investing_activities: Option<f64>,

    /// Net cash provided by or used in financing activities.
    #[serde(rename = "us-gaap:NetCashProvidedByUsedInFinancingActivities", default)]
    pub net_cash_provided_by_financing_activities: Option<f64>,

    /// Net increase or decrease in cash and cash equivalents during the period.
    #[serde(
        rename = "us-gaap:CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalentsPeriodIncreaseDecreaseExcludingExchangeRateEffect",
        default
    )]
    pub cash_and_cash_equivalents_period_increase_decrease: Option<f64>,

    /// Proceeds from initial public offering.
    #[serde(rename = "us-gaap:ProceedsFromIssuanceInitialPublicOffering", default)]
    pub proceeds_from_issuance_initial_public_offering: Option<f64>,

    /// Proceeds from private placement issuance.
    #[serde(rename = "us-gaap:ProceedsFromIssuanceOfPrivatePlacement", default)]
    pub proceeds_from_issuance_of_private_placement: Option<f64>,

    /// Proceeds from notes payable.
    #[serde(rename = "us-gaap:ProceedsFromNotesPayable", default)]
    pub proceeds_from_notes_payable: Option<f64>,

    /// Proceeds from related party debt.
    #[serde(rename = "us-gaap:ProceedsFromRelatedPartyDebt", default)]
    pub proceeds_from_related_party_debt: Option<f64>,

    /// Payments to acquire investments.
    #[serde(rename = "us-gaap:PaymentsToAcquireInvestments", default)]
    pub payments_to_acquire_investments: Option<f64>,

    /// Payments of debt issuance costs.
    #[serde(rename = "us-gaap:PaymentsOfDebtIssuanceCosts", default)]
    pub payments_of_debt_issuance_costs: Option<f64>,

    /// Repayments of related party debt.
    #[serde(rename = "us-gaap:RepaymentsOfRelatedPartyDebt", default)]
    pub repayments_of_related_party_debt: Option<f64>,

    /// Increase or decrease in accrued liabilities.
    #[serde(rename = "us-gaap:IncreaseDecreaseInAccruedLiabilities", default)]
    pub increase_decrease_in_accrued_liabilities: Option<f64>,

    /// Increase or decrease in prepaid expenses.
    #[serde(rename = "us-gaap:IncreaseDecreaseInPrepaidExpense", default)]
    pub increase_decrease_in_prepaid_expense: Option<f64>,

    /// Increase or decrease in accounts payable.
    #[serde(rename = "us-gaap:IncreaseDecreaseInAccountsPayable", default)]
    pub increase_decrease_in_accounts_payable: Option<f64>,

    /// Increase or decrease in deposits outstanding.
    #[serde(rename = "us-gaap:IncreaseDecreaseInDepositsOutstanding", default)]
    pub increase_decrease_in_deposits_outstanding: Option<f64>,

    /// Increase or decrease in amounts due to affiliates.
    #[serde(rename = "us-gaap:IncreaseDecreaseInDueToAffiliates", default)]
    pub increase_decrease_in_due_to_affiliates: Option<f64>,

    /// Period increase/decrease in cash including exchange rate effect.
    #[serde(
        rename = "us-gaap:CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalentsPeriodIncreaseDecreaseIncludingExchangeRateEffect",
        default
    )]
    pub cash_period_increase_decrease_including_fx: Option<f64>,

    /// Increase/decrease in prepaid, deferred expense, and other assets.
    #[serde(
        rename = "us-gaap:IncreaseDecreaseInPrepaidDeferredExpenseAndOtherAssets",
        default
    )]
    pub increase_decrease_prepaid_and_other_assets: Option<f64>,

    /// Increase/decrease in prepaid insurance.
    #[serde(rename = "us-gaap:IncreaseDecreaseInPrepaidInsurance", default)]
    pub increase_decrease_prepaid_insurance: Option<f64>,

    /// Proceeds from issuance of common stock.
    #[serde(rename = "us-gaap:ProceedsFromIssuanceOfCommonStock", default)]
    pub proceeds_from_issuance_of_common_stock: Option<f64>,

    /// Proceeds from issuance of warrants.
    #[serde(rename = "us-gaap:ProceedsFromIssuanceOfWarrants", default)]
    pub proceeds_from_issuance_of_warrants: Option<f64>,

    /// Payments of stock issuance costs.
    #[serde(rename = "us-gaap:PaymentsOfStockIssuanceCosts", default)]
    pub payments_of_stock_issuance_costs: Option<f64>,

    /// Payments for underwriting expense.
    #[serde(rename = "us-gaap:PaymentsForUnderwritingExpense", default)]
    pub payments_for_underwriting_expense: Option<f64>,

    /// Repayments of notes payable.
    #[serde(rename = "us-gaap:RepaymentsOfNotesPayable", default)]
    pub repayments_of_notes_payable: Option<f64>,

    /// Payment of financing and stock issuance costs (combined).
    #[serde(rename = "us-gaap:PaymentOfFinancingAndStockIssuanceCosts", default)]
    pub payment_of_financing_and_stock_issuance_costs: Option<f64>,

    /// Proceeds from stock options exercised.
    #[serde(rename = "us-gaap:ProceedsFromStockOptionsExercised", default)]
    pub proceeds_from_stock_options_exercised: Option<f64>,

    /// Payments for repurchase of common stock.
    #[serde(rename = "us-gaap:PaymentsForRepurchaseOfCommonStock", default)]
    pub payments_for_repurchase_of_common_stock: Option<f64>,
}

/// Represents detailed information about stock issuances and equity transactions.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EquityDetails {
    /// Number of common stock shares authorized.
    #[serde(rename = "us-gaap:CommonStockSharesAuthorized", default)]
    pub common_stock_shares_authorized: Option<f64>,

    /// Number of common stock shares issued.
    #[serde(rename = "us-gaap:CommonStockSharesIssued", default)]
    pub common_stock_shares_issued: Option<f64>,

    /// Number of common stock shares outstanding.
    #[serde(rename = "us-gaap:CommonStockSharesOutstanding", default)]
    pub common_stock_shares_outstanding: Option<f64>,

    /// Par or stated value per common stock share.
    #[serde(rename = "us-gaap:CommonStockParOrStatedValuePerShare", default)]
    pub common_stock_par_value_per_share: Option<f64>,

    /// Voting rights description for common stock.
    #[serde(rename = "us-gaap:CommonStockVotingRights", default)]
    pub common_stock_voting_rights: Option<String>,

    /// Total shares outstanding.
    #[serde(rename = "us-gaap:SharesOutstanding", default)]
    pub shares_outstanding: Option<f64>,

    /// Price per share for shares issued.
    #[serde(rename = "us-gaap:SharesIssuedPricePerShare", default)]
    pub shares_issued_price_per_share: Option<f64>,

    /// Current share price.
    #[serde(rename = "us-gaap:SharePrice", default)]
    pub share_price: Option<f64>,

    /// Price per share for stock sales.
    #[serde(rename = "us-gaap:SaleOfStockPricePerShare", default)]
    pub sale_of_stock_price_per_share: Option<f64>,

    /// Number of shares issued in stock sale transaction.
    #[serde(
        rename = "us-gaap:SaleOfStockNumberOfSharesIssuedInTransaction",
        default
    )]
    pub sale_of_stock_number_of_shares_issued: Option<f64>,

    /// Consideration received from stock sale transaction.
    #[serde(
        rename = "us-gaap:SaleOfStockConsiderationReceivedOnTransaction",
        default
    )]
    pub sale_of_stock_consideration_received_on_transaction: Option<f64>,

    /// Shares issued during period - new issues.
    #[serde(rename = "us-gaap:StockIssuedDuringPeriodSharesNewIssues", default)]
    pub stock_issued_during_period_shares_new_issues: Option<f64>,

    /// Shares issued during period for share-based compensation.
    #[serde(
        rename = "us-gaap:StockIssuedDuringPeriodSharesShareBasedCompensation",
        default
    )]
    pub stock_issued_during_period_shares_share_based_compensation: Option<f64>,

    /// Shares issued during period - conversion of convertible securities.
    #[serde(
        rename = "us-gaap:StockIssuedDuringPeriodSharesConversionOfConvertibleSecurities",
        default
    )]
    pub stock_issued_during_period_shares_conversion_of_convertible_securities: Option<f64>,

    /// Restricted stock awards issued (gross).
    #[serde(
        rename = "us-gaap:StockIssuedDuringPeriodSharesRestrictedStockAwardGross",
        default
    )]
    pub stock_issued_during_period_shares_restricted_stock_award_gross: Option<f64>,

    /// Other shares issued during period.
    #[serde(rename = "us-gaap:StockIssuedDuringPeriodSharesOther", default)]
    pub stock_issued_during_period_shares_other: Option<f64>,

    /// Value of new stock issues during period.
    #[serde(rename = "us-gaap:StockIssuedDuringPeriodValueNewIssues", default)]
    pub stock_issued_during_period_value_new_issues: Option<f64>,

    /// Value of stock issued during period from conversion of convertible securities.
    #[serde(
        rename = "us-gaap:StockIssuedDuringPeriodValueConversionOfConvertibleSecurities",
        default
    )]
    pub stock_issued_during_period_value_conversion_of_convertible_securities: Option<f64>,

    /// Value of share-based compensation forfeited during period.
    #[serde(
        rename = "us-gaap:StockIssuedDuringPeriodValueShareBasedCompensationForfeited",
        default
    )]
    pub stock_issued_during_period_value_share_based_compensation_forfeited: Option<f64>,

    /// Value of other stock issued during period.
    #[serde(rename = "us-gaap:StockIssuedDuringPeriodValueOther", default)]
    pub stock_issued_during_period_value_other: Option<f64>,

    /// General stock issued amount.
    #[serde(rename = "us-gaap:StockIssued1", default)]
    pub stock_issued: Option<f64>,

    /// Shares issued (alternative field).
    #[serde(rename = "us-gaap:SharesIssued", default)]
    pub shares_issued_alt: Option<f64>,

    /// Shares issued during period for services.
    #[serde(
        rename = "us-gaap:StockIssuedDuringPeriodSharesIssuedForServices",
        default
    )]
    pub stock_issued_during_period_shares_issued_for_services: Option<f64>,

    /// Shares forfeited during period (share-based compensation).
    #[serde(
        rename = "us-gaap:StockIssuedDuringPeriodSharesShareBasedCompensationForfeited",
        default
    )]
    pub stock_issued_during_period_shares_share_based_compensation_forfeited: Option<f64>,

    /// Value of stock issued during period for services.
    #[serde(
        rename = "us-gaap:StockIssuedDuringPeriodValueIssuedForServices",
        default
    )]
    pub stock_issued_during_period_value_issued_for_services: Option<f64>,

    /// Shares redeemed or called during period.
    #[serde(rename = "us-gaap:StockRedeemedOrCalledDuringPeriodShares", default)]
    pub stock_redeemed_or_called_during_period_shares: Option<f64>,

    /// Shares repurchased and retired during period.
    #[serde(
        rename = "us-gaap:StockRepurchasedAndRetiredDuringPeriodShares",
        default
    )]
    pub stock_repurchased_and_retired_during_period_shares: Option<f64>,

    /// Value of stock repurchased and retired during period.
    #[serde(
        rename = "us-gaap:StockRepurchasedAndRetiredDuringPeriodValue",
        default
    )]
    pub stock_repurchased_and_retired_during_period_value: Option<f64>,

    /// Amount converted in stock conversion.
    #[serde(rename = "us-gaap:ConversionOfStockAmountConverted1", default)]
    pub conversion_of_stock_amount_converted: Option<f64>,

    /// Adjustments to APIC for warrant issuance.
    #[serde(
        rename = "us-gaap:AdjustmentsToAdditionalPaidInCapitalWarrantIssued",
        default
    )]
    pub adjustments_to_apic_warrant_issued: Option<f64>,

    /// Description of stock sale transaction.
    #[serde(rename = "us-gaap:SaleOfStockDescriptionOfTransaction", default)]
    pub sale_of_stock_description: Option<String>,

    /// Percentage of ownership before stock sale transaction.
    #[serde(
        rename = "us-gaap:SaleOfStockPercentageOfOwnershipBeforeTransaction",
        default
    )]
    pub sale_of_stock_percentage_ownership_before: Option<f64>,

    /// Share price in business acquisition.
    #[serde(rename = "us-gaap:BusinessAcquisitionSharePrice", default)]
    pub business_acquisition_share_price: Option<f64>,
}

/// Represents information about temporary equity and warrants (common in SPACs).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemporaryEquityAndWarrants {
    /// Carrying amount of temporary equity attributable to parent.
    #[serde(
        rename = "us-gaap:TemporaryEquityCarryingAmountAttributableToParent",
        default
    )]
    pub temporary_equity_carrying_amount: Option<f64>,

    /// Accretion of temporary equity to redemption value.
    #[serde(rename = "us-gaap:TemporaryEquityAccretionToRedemptionValue", default)]
    pub temporary_equity_accretion_to_redemption_value: Option<f64>,

    /// Adjustment to temporary equity accretion to redemption value.
    #[serde(
        rename = "us-gaap:TemporaryEquityAccretionToRedemptionValueAdjustment",
        default
    )]
    pub temporary_equity_accretion_adjustment: Option<f64>,

    /// Number of temporary equity shares issued.
    #[serde(rename = "us-gaap:TemporaryEquitySharesIssued", default)]
    pub temporary_equity_shares_issued: Option<f64>,

    /// Number of temporary equity shares outstanding.
    #[serde(rename = "us-gaap:TemporaryEquitySharesOutstanding", default)]
    pub temporary_equity_shares_outstanding: Option<f64>,

    /// Exercise price of warrants or rights.
    #[serde(
        rename = "us-gaap:ClassOfWarrantOrRightExercisePriceOfWarrantsOrRights1",
        default
    )]
    pub warrant_exercise_price: Option<f64>,

    /// Number of securities called by warrants or rights.
    #[serde(
        rename = "us-gaap:ClassOfWarrantOrRightNumberOfSecuritiesCalledByWarrantsOrRights",
        default
    )]
    pub warrant_number_of_securities_called: Option<f64>,

    /// Terms of outstanding warrants and rights.
    #[serde(rename = "us-gaap:WarrantsAndRightsOutstandingTerm", default)]
    pub warrants_and_rights_outstanding_term: Option<String>,

    /// Redemption price per share for temporary equity.
    #[serde(rename = "us-gaap:TemporaryEquityRedemptionPricePerShare", default)]
    pub temporary_equity_redemption_price_per_share: Option<f64>,

    /// Par or stated value per share for temporary equity.
    #[serde(rename = "us-gaap:TemporaryEquityParOrStatedValuePerShare", default)]
    pub temporary_equity_par_or_stated_value_per_share: Option<f64>,

    /// Redemption price per share for preferred stock.
    #[serde(rename = "us-gaap:PreferredStockRedemptionPricePerShare", default)]
    pub preferred_stock_redemption_price_per_share: Option<f64>,

    /// Value of temporary equity stock issued during period (new issues).
    #[serde(
        rename = "us-gaap:TemporaryEquityStockIssuedDuringPeriodValueNewIssues",
        default
    )]
    pub temporary_equity_stock_issued_value_new_issues: Option<f64>,

    /// Number of securities called by each warrant or right.
    #[serde(
        rename = "us-gaap:ClassOfWarrantOrRightNumberOfSecuritiesCalledByEachWarrantOrRight",
        default
    )]
    pub warrant_securities_per_warrant: Option<f64>,

    /// Number of warrants or rights outstanding.
    #[serde(rename = "us-gaap:ClassOfWarrantOrRightOutstanding", default)]
    pub warrants_or_rights_outstanding: Option<f64>,

    /// Warrants and rights outstanding (alternative).
    #[serde(rename = "us-gaap:WarrantsAndRightsOutstanding", default)]
    pub warrants_and_rights_outstanding_count: Option<f64>,

    /// Measurement input for warrants and rights outstanding.
    #[serde(
        rename = "us-gaap:WarrantsAndRightsOutstandingMeasurementInput",
        default
    )]
    pub warrants_measurement_input: Option<f64>,

    /// Valuation technique for warrants and rights (text).
    #[serde(
        rename = "us-gaap:WarrantsAndRightsOutstandingValuationTechniqueExtensibleList",
        default
    )]
    pub warrants_valuation_technique: Option<String>,

    /// Temporary equity table text block (disclosure).
    #[serde(rename = "us-gaap:TemporaryEquityTableTextBlock", default)]
    pub temporary_equity_disclosure: Option<String>,
}

/// Represents acquisition and business combination information (relevant for SPACs).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BusinessCombinations {
    /// Consideration transferred in asset acquisition.
    #[serde(rename = "us-gaap:AssetAcquisitionConsiderationTransferred", default)]
    pub asset_acquisition_consideration_transferred: Option<f64>,

    /// Percentage of voting interests acquired in business acquisition.
    #[serde(
        rename = "us-gaap:BusinessAcquisitionPercentageOfVotingInterestsAcquired",
        default
    )]
    pub business_acquisition_percentage_of_voting_interests: Option<f64>,

    /// Percentage of equity interest in step acquisition.
    #[serde(
        rename = "us-gaap:BusinessCombinationStepAcquisitionEquityInterestInAcquireePercentage",
        default
    )]
    pub step_acquisition_equity_interest_percentage: Option<f64>,

    /// Fair value of equity issued in business combination.
    #[serde(
        rename = "us-gaap:EquityIssuedInBusinessCombinationFairValueDisclosure",
        default
    )]
    pub equity_issued_fair_value: Option<f64>,

    /// Supplemental deferred purchase price.
    #[serde(rename = "us-gaap:SupplementalDeferredPurchasePrice", default)]
    pub supplemental_deferred_purchase_price: Option<f64>,

    /// Segment allocation table for business combination (text block).
    #[serde(
        rename = "us-gaap:BusinessCombinationSegmentAllocationTableTextBlock",
        default
    )]
    pub business_combination_segment_allocation_table: Option<String>,
}

/// Represents debt instruments and conversion details.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DebtDetails {
    /// Face amount of debt instrument.
    #[serde(rename = "us-gaap:DebtInstrumentFaceAmount", default)]
    pub debt_instrument_face_amount: Option<f64>,

    /// Carrying amount of debt instrument.
    #[serde(rename = "us-gaap:DebtInstrumentCarryingAmount", default)]
    pub debt_instrument_carrying_amount: Option<f64>,

    /// Stated interest rate percentage on debt instrument.
    #[serde(rename = "us-gaap:DebtInstrumentInterestRateStatedPercentage", default)]
    pub debt_instrument_interest_rate_stated_percentage: Option<f64>,

    /// Conversion price of convertible debt instrument.
    #[serde(rename = "us-gaap:DebtInstrumentConvertibleConversionPrice1", default)]
    pub debt_convertible_conversion_price: Option<f64>,

    /// Amount of converted instrument in debt conversion.
    #[serde(rename = "us-gaap:DebtConversionConvertedInstrumentAmount1", default)]
    pub debt_conversion_converted_amount: Option<f64>,

    /// Debt issuance costs incurred during noncash or partial noncash transaction.
    #[serde(
        rename = "us-gaap:DebtIssuanceCostsIncurredDuringNoncashOrPartialNoncashTransaction",
        default
    )]
    pub debt_issuance_costs_incurred_during_noncash_transaction: Option<f64>,

    /// Notes issued value.
    #[serde(rename = "us-gaap:NotesIssued1", default)]
    pub notes_issued: Option<f64>,

    /// Extensible enumeration for related party notes payable type.
    #[serde(
        rename = "us-gaap:NotesPayableCurrentRelatedPartyTypeExtensibleEnumeration",
        default
    )]
    pub notes_payable_related_party_type: Option<String>,
}

/// Represents cash management and FDIC insurance details.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CashManagement {
    /// Amount of cash that is FDIC insured.
    #[serde(rename = "us-gaap:CashFDICInsuredAmount", default)]
    pub cash_fdic_insured_amount: Option<f64>,

    /// Federal Deposit Insurance Corporation premium expense.
    #[serde(
        rename = "us-gaap:FederalDepositInsuranceCorporationPremiumExpense",
        default
    )]
    pub fdic_premium_expense: Option<f64>,
}

/// Represents commitments and contingencies placeholder.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommitmentsAndContingencies {
    /// Commitments and contingencies placeholder value.
    #[serde(rename = "us-gaap:CommitmentsAndContingencies", default)]
    pub commitments_and_contingencies: Option<f64>,
}

/// Represents segment and related party information.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SegmentAndRelatedParty {
    /// Number of operating segments.
    #[serde(rename = "us-gaap:NumberOfOperatingSegments", default)]
    pub number_of_operating_segments: Option<i32>,

    /// Number of reportable segments.
    #[serde(rename = "us-gaap:NumberOfReportableSegments", default)]
    pub number_of_reportable_segments: Option<i32>,

    /// Amount of related party transaction.
    #[serde(
        rename = "us-gaap:RelatedPartyTransactionAmountsOfTransaction",
        default
    )]
    pub related_party_transaction_amount: Option<f64>,

    /// Extensible enumeration for segment reporting CODM title.
    #[serde(
        rename = "us-gaap:SegmentReportingCodmIndividualTitleAndPositionOrGroupOrCommitteeNameExtensibleEnumeration",
        default
    )]
    pub segment_codm_title: Option<String>,
}

/// Represents tax-related disclosures.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaxDetails {
    /// Unrecognized tax benefits.
    #[serde(rename = "us-gaap:UnrecognizedTaxBenefits", default)]
    pub unrecognized_tax_benefits: Option<f64>,

    /// Accrued interest and penalties on unrecognized tax benefits.
    #[serde(
        rename = "us-gaap:UnrecognizedTaxBenefitsIncomeTaxPenaltiesAndInterestAccrued",
        default
    )]
    pub unrecognized_tax_benefits_penalties_and_interest: Option<f64>,
}

/// Represents risk concentrations and accounting policies.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RisksAndPolicies {
    /// Concentration risk related to credit risk.
    #[serde(rename = "us-gaap:ConcentrationRiskCreditRisk", default)]
    pub concentration_risk_credit: Option<String>,

    /// Use of estimates policy narrative.
    #[serde(rename = "us-gaap:UseOfEstimates", default)]
    pub use_of_estimates: Option<String>,

    /// Dilutive securities disclosure.
    #[serde(rename = "us-gaap:DilutiveSecurities", default)]
    pub dilutive_securities: Option<String>,
}

/// Represents comprehensive income components.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComprehensiveIncomeDetails {
    /// Other comprehensive income from unrealized gains/losses on securities.
    #[serde(
        rename = "us-gaap:OtherComprehensiveIncomeUnrealizedHoldingGainLossOnSecuritiesArisingDuringPeriodBeforeTax",
        default
    )]
    pub other_comprehensive_income_unrealized_securities: Option<f64>,

    /// Other comprehensive income from foreign currency transactions and translations.
    #[serde(
        rename = "us-gaap:OtherComprehensiveIncomeLossForeignCurrencyTransactionAndTranslationReclassificationAdjustmentFromAOCIRealizedUponSaleOrLiquidationBeforeTax",
        default
    )]
    pub other_comprehensive_income_foreign_currency: Option<f64>,

    /// Other comprehensive income reclassification for held-to-maturity transfers.
    #[serde(
        rename = "us-gaap:OtherComprehensiveIncomeReclassificationAdjustmentForHeldToMaturityTransferredToAvailableForSaleSecuritiesBeforeTax",
        default
    )]
    pub other_comprehensive_income_htm_reclassification: Option<f64>,

    /// Adjustments to additional paid-in capital for stock issuance costs.
    #[serde(
        rename = "us-gaap:AdjustmentsToAdditionalPaidInCapitalStockIssuedIssuanceCosts",
        default
    )]
    pub adjustments_to_paid_in_capital_issuance_costs: Option<f64>,
}

/// Represents key narrative text blocks for LLM analysis.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Narratives {
    /// Description of the nature of the entity's operations.
    #[serde(rename = "us-gaap:NatureOfOperations", default)]
    pub nature_of_operations: Option<String>,

    /// Disclosure text block for commitments and contingencies.
    #[serde(
        rename = "us-gaap:CommitmentsAndContingenciesDisclosureTextBlock",
        default
    )]
    pub commitments_and_contingencies: Option<String>,

    /// Text block describing significant accounting policies.
    #[serde(rename = "us-gaap:SignificantAccountingPoliciesTextBlock", default)]
    pub significant_accounting_policies: Option<String>,

    /// Text block describing consolidation policy.
    #[serde(rename = "us-gaap:ConsolidationPolicyTextBlock", default)]
    pub consolidation_policy: Option<String>,

    /// Combined basis of presentation and significant accounting policies text block.
    #[serde(
        rename = "us-gaap:BasisOfPresentationAndSignificantAccountingPoliciesTextBlock",
        default
    )]
    pub basis_of_presentation_and_significant_accounting_policies: Option<String>,

    /// Text block describing subsequent events.
    #[serde(rename = "us-gaap:SubsequentEventsTextBlock", default)]
    pub subsequent_events: Option<String>,

    /// Text block describing cash and cash equivalents policy.
    #[serde(rename = "us-gaap:CashAndCashEquivalentsPolicyTextBlock", default)]
    pub cash_and_cash_equivalents_policy: Option<String>,

    /// Text block describing earnings per share policy.
    #[serde(rename = "us-gaap:EarningsPerSharePolicyTextBlock", default)]
    pub earnings_per_share_policy: Option<String>,

    /// Policy for fair value of financial instruments.
    #[serde(rename = "us-gaap:FairValueOfFinancialInstrumentsPolicy", default)]
    pub fair_value_of_financial_instruments_policy: Option<String>,

    /// Text block describing income tax policy.
    #[serde(rename = "us-gaap:IncomeTaxPolicyTextBlock", default)]
    pub income_tax_policy: Option<String>,

    /// Text block describing derivatives policy.
    #[serde(rename = "us-gaap:DerivativesPolicyTextBlock", default)]
    pub derivatives_policy: Option<String>,

    /// Text block describing investment policy.
    #[serde(rename = "us-gaap:InvestmentPolicyTextBlock", default)]
    pub investment_policy: Option<String>,

    /// Text block describing new accounting pronouncements policy.
    #[serde(
        rename = "us-gaap:NewAccountingPronouncementsPolicyPolicyTextBlock",
        default
    )]
    pub new_accounting_pronouncements_policy: Option<String>,

    /// Text block disclosing related party transactions.
    #[serde(
        rename = "us-gaap:RelatedPartyTransactionsDisclosureTextBlock",
        default
    )]
    pub related_party_transactions: Option<String>,

    /// Text block for stockholders' equity note disclosure.
    #[serde(rename = "us-gaap:StockholdersEquityNoteDisclosureTextBlock", default)]
    pub stockholders_equity_note: Option<String>,

    /// Text block for fair value assets measured on recurring basis.
    #[serde(
        rename = "us-gaap:FairValueAssetsMeasuredOnRecurringBasisTextBlock",
        default
    )]
    pub fair_value_assets_measured_recurring: Option<String>,

    /// Basis of accounting policy text block.
    #[serde(rename = "us-gaap:BasisOfAccountingPolicyPolicyTextBlock", default)]
    pub basis_of_accounting_policy: Option<String>,

    /// Deferred charges policy text block.
    #[serde(rename = "us-gaap:DeferredChargesPolicyTextBlock", default)]
    pub deferred_charges_policy: Option<String>,

    /// Fair value assets and liabilities valuation techniques table text block.
    #[serde(
        rename = "us-gaap:FairValueAssetsAndLiabilitiesMeasuredOnRecurringAndNonrecurringBasisValuationTechniquesTableTextBlock",
        default
    )]
    pub fair_value_valuation_techniques_table: Option<String>,

    /// Fair value disclosures text block.
    #[serde(rename = "us-gaap:FairValueDisclosuresTextBlock", default)]
    pub fair_value_disclosures: Option<String>,

    /// Fair value measurement policy text block.
    #[serde(rename = "us-gaap:FairValueMeasurementPolicyPolicyTextBlock", default)]
    pub fair_value_measurement_policy: Option<String>,

    /// Marketable securities policy text block.
    #[serde(rename = "us-gaap:MarketableSecuritiesPolicy", default)]
    pub marketable_securities_policy: Option<String>,

    /// Organization, consolidation, and presentation of financial statements disclosure text block.
    #[serde(
        rename = "us-gaap:OrganizationConsolidationAndPresentationOfFinancialStatementsDisclosureTextBlock",
        default
    )]
    pub organization_consolidation_presentation_disclosure: Option<String>,

    /// Reconciliation of assets from segment to consolidated text block.
    #[serde(
        rename = "us-gaap:ReconciliationOfAssetsFromSegmentToConsolidatedTextBlock",
        default
    )]
    pub reconciliation_assets_segment_to_consolidated: Option<String>,

    /// Schedule of segment reporting information by segment text block.
    #[serde(
        rename = "us-gaap:ScheduleOfSegmentReportingInformationBySegmentTextBlock",
        default
    )]
    pub schedule_segment_reporting_information: Option<String>,

    /// Description of how CODM profit/loss measure is used.
    #[serde(
        rename = "us-gaap:SegmentReportingCodmProfitLossMeasureHowUsedDescription",
        default
    )]
    pub segment_codm_profit_loss_measure_description: Option<String>,

    /// Share-based compensation option and incentive plans policy text block.
    #[serde(
        rename = "us-gaap:ShareBasedCompensationOptionAndIncentivePlansPolicy",
        default
    )]
    pub share_based_compensation_policy: Option<String>,

    /// Text block for schedule of earnings per share basic and diluted.
    #[serde(
        rename = "us-gaap:ScheduleOfEarningsPerShareBasicAndDilutedTableTextBlock",
        default
    )]
    pub schedule_of_earnings_per_share: Option<String>,

    /// Text block for segment reporting disclosure.
    #[serde(rename = "us-gaap:SegmentReportingDisclosureTextBlock", default)]
    pub segment_reporting: Option<String>,

    /// Text block describing subsidiary of limited liability company or limited partnership.
    #[serde(
        rename = "us-gaap:ScheduleOfSubsidiaryOfLimitedLiabilityCompanyOrLimitedPartnershipDescriptionTextBlock",
        default
    )]
    pub subsidiary_description: Option<String>,

    /// Text block for shares subject to mandatory redemption policy.
    #[serde(
        rename = "us-gaap:SharesSubjectToMandatoryRedemptionChangesInRedemptionValuePolicyTextBlock",
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
