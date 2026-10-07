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

use crate::FromXbrl;
use crate::bind::XbrlDataContext;
use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Represents key data points from the Balance Sheet.
#[derive(Debug, Clone, Default, Serialize, Deserialize, FromXbrl)]
#[serde(default)]
#[xbrl(instant)]
pub struct BalanceSheet {
    /// The date these figures are as of.
    ///
    /// Set on each of [`Financials::balance_sheets`]. `None` on
    /// [`Financials::balance_sheet`], whose fields are each the best the filing
    /// offers and can come from different dates.
    #[xbrl(period_end)]
    pub as_of: Option<String>,

    // --- Assets Section ---
    /// Total assets of the entity.
    #[xbrl(concept = "us-gaap:Assets")]
    pub assets: Option<f64>,

    /// Current assets expected to be realized within one year.
    #[xbrl(concept = "us-gaap:AssetsCurrent")]
    pub assets_current: Option<f64>,

    /// Assets held in trust (common in SPACs).
    #[xbrl(concept = "us-gaap:AssetsHeldInTrust")]
    pub assets_held_in_trust: Option<f64>,

    /// Non-current assets held in trust.
    #[xbrl(concept = "us-gaap:AssetsHeldInTrustNoncurrent")]
    pub assets_held_in_trust_noncurrent: Option<f64>,

    /// Cash (not including cash equivalents).
    #[xbrl(concept = "us-gaap:Cash")]
    pub cash: Option<f64>,

    /// Cash and cash equivalents at carrying value.
    #[xbrl(concept = "us-gaap:CashAndCashEquivalentsAtCarryingValue")]
    pub cash_and_cash_equivalents: Option<f64>,

    /// Cash equivalents at carrying value.
    #[xbrl(concept = "us-gaap:CashEquivalentsAtCarryingValue")]
    pub cash_equivalents_at_carrying_value: Option<f64>,

    /// Total of cash, cash equivalents, restricted cash, and restricted cash equivalents.
    #[xbrl(concept = "us-gaap:CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalents")]
    pub cash_cash_equivalents_restricted_cash: Option<f64>,

    /// Marketable securities classified as noncurrent.
    #[xbrl(concept = "us-gaap:MarketableSecuritiesNoncurrent")]
    pub marketable_securities_noncurrent: Option<f64>,

    /// Prepaid expenses expected to be consumed within one year.
    #[xbrl(concept = "us-gaap:PrepaidExpenseCurrent")]
    pub prepaid_expense_current: Option<f64>,

    /// Prepaid expenses expected to be consumed beyond one year.
    #[xbrl(concept = "us-gaap:PrepaidExpenseNoncurrent")]
    pub prepaid_expense_noncurrent: Option<f64>,

    /// Prepaid insurance premiums.
    #[xbrl(concept = "us-gaap:PrepaidInsurance")]
    pub prepaid_insurance: Option<f64>,

    /// Other prepaid expenses not elsewhere classified.
    #[xbrl(concept = "us-gaap:OtherPrepaidExpenseCurrent")]
    pub other_prepaid_expense_current: Option<f64>,

    /// Costs deferred in connection with public or private offerings.
    #[xbrl(concept = "us-gaap:DeferredOfferingCosts")]
    pub deferred_offering_costs: Option<f64>,

    /// Deferred costs not elsewhere classified.
    #[xbrl(concept = "us-gaap:DeferredCosts")]
    pub deferred_costs: Option<f64>,

    /// Assets held as deposits.
    #[xbrl(concept = "us-gaap:DepositAssets")]
    pub deposit_assets: Option<f64>,

    /// Other receivables not elsewhere classified.
    #[xbrl(concept = "us-gaap:OtherReceivables")]
    pub other_receivables: Option<f64>,

    /// Other prepaid expenses classified as noncurrent.
    #[xbrl(concept = "us-gaap:PrepaidExpenseOtherNoncurrent")]
    pub prepaid_expense_other_noncurrent: Option<f64>,

    // --- Liabilities Section ---
    /// Total liabilities of the entity.
    #[xbrl(concept = "us-gaap:Liabilities")]
    pub liabilities: Option<f64>,

    /// Current liabilities expected to be settled within one year.
    #[xbrl(concept = "us-gaap:LiabilitiesCurrent")]
    pub liabilities_current: Option<f64>,

    /// Amounts owed to trade creditors within one year.
    #[xbrl(concept = "us-gaap:AccountsPayableCurrent")]
    pub accounts_payable_current: Option<f64>,

    /// Accrued liabilities due within one year.
    #[xbrl(concept = "us-gaap:AccruedLiabilitiesCurrent")]
    pub accrued_liabilities_current: Option<f64>,

    /// Other current liabilities not elsewhere classified.
    #[xbrl(concept = "us-gaap:OtherLiabilitiesCurrent")]
    pub other_liabilities_current: Option<f64>,

    /// Other liabilities not elsewhere classified (non-current).
    #[xbrl(concept = "us-gaap:OtherLiabilities")]
    pub other_liabilities: Option<f64>,

    /// Notes payable.
    #[xbrl(concept = "us-gaap:NotesPayable")]
    pub notes_payable: Option<f64>,

    /// Notes payable due within one year.
    #[xbrl(concept = "us-gaap:NotesPayableCurrent")]
    pub notes_payable_current: Option<f64>,

    /// Long-term notes payable.
    #[xbrl(concept = "us-gaap:LongTermNotesPayable")]
    pub long_term_notes_payable: Option<f64>,

    /// Loans payable.
    #[xbrl(concept = "us-gaap:LoansPayable")]
    pub loans_payable: Option<f64>,

    /// Short-term borrowings.
    #[xbrl(concept = "us-gaap:ShortTermBorrowings")]
    pub short_term_borrowings: Option<f64>,

    /// Unsecured debt.
    #[xbrl(concept = "us-gaap:UnsecuredDebt")]
    pub unsecured_debt: Option<f64>,

    /// Bank overdrafts.
    #[xbrl(concept = "us-gaap:BankOverdrafts")]
    pub bank_overdrafts: Option<f64>,

    /// Other notes payable due within one year.
    #[xbrl(concept = "us-gaap:OtherNotesPayableCurrent")]
    pub other_notes_payable_current: Option<f64>,

    /// Other notes payable (general, not classified by maturity).
    #[xbrl(concept = "us-gaap:OtherNotesPayable")]
    pub other_notes_payable: Option<f64>,

    /// Interest payable, current and noncurrent.
    #[xbrl(concept = "us-gaap:InterestPayableCurrentAndNoncurrent")]
    pub interest_payable_current_and_noncurrent: Option<f64>,

    /// Long-term debt.
    #[xbrl(concept = "us-gaap:LongTermDebt")]
    pub long_term_debt: Option<f64>,

    /// Deferred compensation liability classified as non-current.
    #[xbrl(concept = "us-gaap:DeferredCompensationLiabilityClassifiedNoncurrent")]
    pub deferred_compensation_liability_noncurrent: Option<f64>,

    // --- Equity Section ---
    /// Total stockholders' equity.
    #[xbrl(concept = "us-gaap:StockholdersEquity")]
    pub stockholders_equity: Option<f64>,

    /// Par or stated value of common stock issued.
    #[xbrl(concept = "us-gaap:CommonStockValue")]
    pub common_stock_value: Option<f64>,

    /// Preferred stock value.
    #[xbrl(concept = "us-gaap:PreferredStockValue")]
    pub preferred_stock_value: Option<f64>,

    /// Preferred stock par value per share.
    #[xbrl(concept = "us-gaap:PreferredStockParOrStatedValuePerShare")]
    pub preferred_stock_par_value_per_share: Option<f64>,

    /// Number of preferred shares authorized.
    #[xbrl(concept = "us-gaap:PreferredStockSharesAuthorized")]
    pub preferred_stock_shares_authorized: Option<f64>,

    /// Number of preferred shares issued.
    #[xbrl(concept = "us-gaap:PreferredStockSharesIssued")]
    pub preferred_stock_shares_issued: Option<f64>,

    /// Number of preferred shares outstanding.
    #[xbrl(concept = "us-gaap:PreferredStockSharesOutstanding")]
    pub preferred_stock_shares_outstanding: Option<f64>,

    /// Additional paid-in capital from stock issuances.
    #[xbrl(concept = "us-gaap:AdditionalPaidInCapital")]
    pub additional_paid_in_capital: Option<f64>,

    /// Retained earnings or accumulated deficit.
    #[xbrl(concept = "us-gaap:RetainedEarningsAccumulatedDeficit")]
    pub retained_earnings_accumulated_deficit: Option<f64>,

    /// Accumulated other comprehensive income or loss, net of tax.
    #[xbrl(concept = "us-gaap:AccumulatedOtherComprehensiveIncomeLossNetOfTax")]
    pub accumulated_other_comprehensive_income_loss: Option<f64>,

    /// Total liabilities and stockholders' equity (should equal total assets).
    #[xbrl(concept = "us-gaap:LiabilitiesAndStockholdersEquity")]
    pub liabilities_and_stockholders_equity: Option<f64>,
}

/// Represents key data points from the Income Statement.
#[derive(Debug, Clone, Default, Serialize, Deserialize, FromXbrl)]
#[serde(default)]
#[xbrl(duration)]
pub struct IncomeStatement {
    /// First day of the period these figures cover.
    ///
    /// Set, with `period_end`, on each of [`Financials::income_statements`]. `None` on
    /// [`Financials::income_statement`], whose fields are each the best the filing
    /// offers and can come from different periods.
    #[xbrl(period_start)]
    pub period_start: Option<String>,

    /// Last day of the period these figures cover.
    #[xbrl(period_end)]
    pub period_end: Option<String>,

    /// Total revenues recognized during the period.
    #[xbrl(concept = "us-gaap:Revenues")]
    pub revenues: Option<f64>,

    /// Operating income or loss.
    #[xbrl(concept = "us-gaap:OperatingIncomeLoss")]
    pub operating_income_loss: Option<f64>,

    /// Total operating expenses incurred during the period.
    #[xbrl(concept = "us-gaap:OperatingExpenses")]
    pub operating_expenses: Option<f64>,

    /// Total operating costs and expenses.
    #[xbrl(concept = "us-gaap:OperatingCostsAndExpenses")]
    pub operating_costs_and_expenses: Option<f64>,

    /// General and administrative expenses.
    #[xbrl(concept = "us-gaap:GeneralAndAdministrativeExpense")]
    pub general_and_administrative_expense: Option<f64>,

    /// Administrative fees expense.
    #[xbrl(concept = "us-gaap:AdministrativeFeesExpense")]
    pub administrative_fees_expense: Option<f64>,

    /// Professional and contract services expense.
    #[xbrl(concept = "us-gaap:ProfessionalAndContractServicesExpense")]
    pub professional_and_contract_services_expense: Option<f64>,

    /// Professional fees.
    #[xbrl(concept = "us-gaap:ProfessionalFees")]
    pub professional_fees: Option<f64>,

    /// Travel and entertainment expense.
    #[xbrl(concept = "us-gaap:TravelAndEntertainmentExpense")]
    pub travel_and_entertainment_expense: Option<f64>,

    /// SPAC sponsor fees.
    #[xbrl(concept = "us-gaap:SponsorFees")]
    pub sponsor_fees: Option<f64>,

    /// Income tax expense or benefit.
    #[xbrl(concept = "us-gaap:IncomeTaxExpenseBenefit")]
    pub income_tax_expense_benefit: Option<f64>,

    /// Utilities operating expenses for products and services.
    #[xbrl(concept = "us-gaap:UtilitiesOperatingExpenseProductsAndServices")]
    pub utilities_operating_expense: Option<f64>,

    /// Non-operating income and expenses.
    #[xbrl(concept = "us-gaap:NonoperatingIncomeExpense")]
    pub nonoperating_income_expense: Option<f64>,

    /// Non-operating interest expense.
    #[xbrl(concept = "us-gaap:InterestExpenseNonoperating")]
    pub interest_expense_nonoperating: Option<f64>,

    /// Interest and other income.
    #[xbrl(concept = "us-gaap:InterestAndOtherIncome")]
    pub interest_and_other_income: Option<f64>,

    /// Other interest income.
    #[xbrl(concept = "us-gaap:InterestIncomeOther")]
    pub interest_income_other: Option<f64>,

    /// Investment income from interest.
    #[xbrl(concept = "us-gaap:InvestmentIncomeInterest")]
    pub investment_income_interest: Option<f64>,

    /// Investment income from dividends.
    #[xbrl(concept = "us-gaap:InvestmentIncomeDividend")]
    pub investment_income_dividend: Option<f64>,

    /// Gain or loss on investments.
    #[xbrl(concept = "us-gaap:GainLossOnInvestments")]
    pub gain_loss_on_investments: Option<f64>,

    /// Fair value adjustment of warrants.
    #[xbrl(concept = "us-gaap:FairValueAdjustmentOfWarrants")]
    pub fair_value_adjustment_of_warrants: Option<f64>,

    /// Other underwriting expenses.
    #[xbrl(concept = "us-gaap:OtherUnderwritingExpense")]
    pub other_underwriting_expense: Option<f64>,

    /// Payments for fees.
    #[xbrl(concept = "us-gaap:PaymentsForFees")]
    pub payments_for_fees: Option<f64>,

    /// Net income or loss for the period.
    #[xbrl(concept = "us-gaap:NetIncomeLoss")]
    pub net_income_loss: Option<f64>,

    /// Profit or loss (alternative name for net income/loss).
    #[xbrl(concept = "us-gaap:ProfitLoss")]
    pub profit_loss: Option<f64>,

    /// Net income or loss available to common stockholders (basic).
    #[xbrl(concept = "us-gaap:NetIncomeLossAvailableToCommonStockholdersBasic")]
    pub net_income_loss_available_to_common_stockholders_basic: Option<f64>,

    /// Net income or loss available to common stockholders (diluted).
    #[xbrl(concept = "us-gaap:NetIncomeLossAvailableToCommonStockholdersDiluted")]
    pub net_income_loss_available_to_common_stockholders_diluted: Option<f64>,

    /// Comprehensive income, net of tax.
    #[xbrl(concept = "us-gaap:ComprehensiveIncomeNetOfTax")]
    pub comprehensive_income_net_of_tax: Option<f64>,

    /// Basic earnings per share.
    #[xbrl(concept = "us-gaap:EarningsPerShareBasic")]
    pub earnings_per_share_basic: Option<f64>,

    /// Diluted earnings per share.
    #[xbrl(concept = "us-gaap:EarningsPerShareDiluted")]
    pub earnings_per_share_diluted: Option<f64>,

    /// Weighted average number of shares outstanding (basic).
    #[xbrl(concept = "us-gaap:WeightedAverageNumberOfSharesOutstandingBasic")]
    pub weighted_average_shares_outstanding_basic: Option<f64>,

    /// Weighted average number of diluted shares outstanding.
    #[xbrl(concept = "us-gaap:WeightedAverageNumberOfDilutedSharesOutstanding")]
    pub weighted_average_shares_outstanding_diluted: Option<f64>,

    /// Antidilutive securities excluded from computation of earnings per share.
    #[xbrl(
        concept = "us-gaap:AntidilutiveSecuritiesExcludedFromComputationOfEarningsPerShareAmount"
    )]
    pub antidilutive_securities_excluded_from_eps: Option<f64>,
}

/// Represents key data points from the Statement of Cash Flows.
#[derive(Debug, Clone, Default, Serialize, Deserialize, FromXbrl)]
#[serde(default)]
#[xbrl(duration)]
pub struct CashFlowStatement {
    /// First day of the period these figures cover.
    ///
    /// Set, with `period_end`, on each of [`Financials::cash_flow_statements`]. `None` on
    /// [`Financials::cash_flow_statement`], whose fields are each the best the filing
    /// offers and can come from different periods.
    #[xbrl(period_start)]
    pub period_start: Option<String>,

    /// Last day of the period these figures cover.
    #[xbrl(period_end)]
    pub period_end: Option<String>,

    /// Net cash provided by or used in operating activities.
    #[xbrl(concept = "us-gaap:NetCashProvidedByUsedInOperatingActivities")]
    pub net_cash_provided_by_operating_activities: Option<f64>,

    /// Net cash provided by or used in investing activities.
    #[xbrl(concept = "us-gaap:NetCashProvidedByUsedInInvestingActivities")]
    pub net_cash_provided_by_investing_activities: Option<f64>,

    /// Net cash provided by or used in financing activities.
    #[xbrl(concept = "us-gaap:NetCashProvidedByUsedInFinancingActivities")]
    pub net_cash_provided_by_financing_activities: Option<f64>,

    /// Net increase or decrease in cash and cash equivalents during the period.
    #[xbrl(
        concept = "us-gaap:CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalentsPeriodIncreaseDecreaseExcludingExchangeRateEffect"
    )]
    pub cash_and_cash_equivalents_period_increase_decrease: Option<f64>,

    /// Proceeds from initial public offering.
    #[xbrl(concept = "us-gaap:ProceedsFromIssuanceInitialPublicOffering")]
    pub proceeds_from_issuance_initial_public_offering: Option<f64>,

    /// Proceeds from private placement issuance.
    #[xbrl(concept = "us-gaap:ProceedsFromIssuanceOfPrivatePlacement")]
    pub proceeds_from_issuance_of_private_placement: Option<f64>,

    /// Proceeds from notes payable.
    #[xbrl(concept = "us-gaap:ProceedsFromNotesPayable")]
    pub proceeds_from_notes_payable: Option<f64>,

    /// Proceeds from related party debt.
    #[xbrl(concept = "us-gaap:ProceedsFromRelatedPartyDebt")]
    pub proceeds_from_related_party_debt: Option<f64>,

    /// Payments to acquire investments.
    #[xbrl(concept = "us-gaap:PaymentsToAcquireInvestments")]
    pub payments_to_acquire_investments: Option<f64>,

    /// Payments of debt issuance costs.
    #[xbrl(concept = "us-gaap:PaymentsOfDebtIssuanceCosts")]
    pub payments_of_debt_issuance_costs: Option<f64>,

    /// Repayments of related party debt.
    #[xbrl(concept = "us-gaap:RepaymentsOfRelatedPartyDebt")]
    pub repayments_of_related_party_debt: Option<f64>,

    /// Increase or decrease in accrued liabilities.
    #[xbrl(concept = "us-gaap:IncreaseDecreaseInAccruedLiabilities")]
    pub increase_decrease_in_accrued_liabilities: Option<f64>,

    /// Increase or decrease in prepaid expenses.
    #[xbrl(concept = "us-gaap:IncreaseDecreaseInPrepaidExpense")]
    pub increase_decrease_in_prepaid_expense: Option<f64>,

    /// Increase or decrease in accounts payable.
    #[xbrl(concept = "us-gaap:IncreaseDecreaseInAccountsPayable")]
    pub increase_decrease_in_accounts_payable: Option<f64>,

    /// Increase or decrease in deposits outstanding.
    #[xbrl(concept = "us-gaap:IncreaseDecreaseInDepositsOutstanding")]
    pub increase_decrease_in_deposits_outstanding: Option<f64>,

    /// Increase or decrease in amounts due to affiliates.
    #[xbrl(concept = "us-gaap:IncreaseDecreaseInDueToAffiliates")]
    pub increase_decrease_in_due_to_affiliates: Option<f64>,

    /// Period increase/decrease in cash including exchange rate effect.
    #[xbrl(
        concept = "us-gaap:CashCashEquivalentsRestrictedCashAndRestrictedCashEquivalentsPeriodIncreaseDecreaseIncludingExchangeRateEffect"
    )]
    pub cash_period_increase_decrease_including_fx: Option<f64>,

    /// Increase/decrease in prepaid, deferred expense, and other assets.
    #[xbrl(concept = "us-gaap:IncreaseDecreaseInPrepaidDeferredExpenseAndOtherAssets")]
    pub increase_decrease_prepaid_and_other_assets: Option<f64>,

    /// Increase/decrease in prepaid insurance.
    #[xbrl(concept = "us-gaap:IncreaseDecreaseInPrepaidInsurance")]
    pub increase_decrease_prepaid_insurance: Option<f64>,

    /// Proceeds from issuance of common stock.
    #[xbrl(concept = "us-gaap:ProceedsFromIssuanceOfCommonStock")]
    pub proceeds_from_issuance_of_common_stock: Option<f64>,

    /// Proceeds from issuance of warrants.
    #[xbrl(concept = "us-gaap:ProceedsFromIssuanceOfWarrants")]
    pub proceeds_from_issuance_of_warrants: Option<f64>,

    /// Payments of stock issuance costs.
    #[xbrl(concept = "us-gaap:PaymentsOfStockIssuanceCosts")]
    pub payments_of_stock_issuance_costs: Option<f64>,

    /// Payments for underwriting expense.
    #[xbrl(concept = "us-gaap:PaymentsForUnderwritingExpense")]
    pub payments_for_underwriting_expense: Option<f64>,

    /// Repayments of notes payable.
    #[xbrl(concept = "us-gaap:RepaymentsOfNotesPayable")]
    pub repayments_of_notes_payable: Option<f64>,

    /// Payment of financing and stock issuance costs (combined).
    #[xbrl(concept = "us-gaap:PaymentOfFinancingAndStockIssuanceCosts")]
    pub payment_of_financing_and_stock_issuance_costs: Option<f64>,

    /// Proceeds from stock options exercised.
    #[xbrl(concept = "us-gaap:ProceedsFromStockOptionsExercised")]
    pub proceeds_from_stock_options_exercised: Option<f64>,

    /// Payments for repurchase of common stock.
    #[xbrl(concept = "us-gaap:PaymentsForRepurchaseOfCommonStock")]
    pub payments_for_repurchase_of_common_stock: Option<f64>,
}

/// Represents detailed information about stock issuances and equity transactions.
#[derive(Debug, Clone, Default, Serialize, Deserialize, FromXbrl)]
#[serde(default)]
pub struct EquityDetails {
    /// Number of common stock shares authorized.
    #[xbrl(concept = "us-gaap:CommonStockSharesAuthorized")]
    pub common_stock_shares_authorized: Option<f64>,

    /// Number of common stock shares issued.
    #[xbrl(concept = "us-gaap:CommonStockSharesIssued")]
    pub common_stock_shares_issued: Option<f64>,

    /// Number of common stock shares outstanding.
    #[xbrl(concept = "us-gaap:CommonStockSharesOutstanding")]
    pub common_stock_shares_outstanding: Option<f64>,

    /// Par or stated value per common stock share.
    #[xbrl(concept = "us-gaap:CommonStockParOrStatedValuePerShare")]
    pub common_stock_par_value_per_share: Option<f64>,

    /// Voting rights description for common stock.
    #[xbrl(concept = "us-gaap:CommonStockVotingRights")]
    pub common_stock_voting_rights: Option<String>,

    /// Total shares outstanding.
    #[xbrl(concept = "us-gaap:SharesOutstanding")]
    pub shares_outstanding: Option<f64>,

    /// Price per share for shares issued.
    #[xbrl(concept = "us-gaap:SharesIssuedPricePerShare")]
    pub shares_issued_price_per_share: Option<f64>,

    /// Current share price.
    #[xbrl(concept = "us-gaap:SharePrice")]
    pub share_price: Option<f64>,

    /// Price per share for stock sales.
    #[xbrl(concept = "us-gaap:SaleOfStockPricePerShare")]
    pub sale_of_stock_price_per_share: Option<f64>,

    /// Number of shares issued in stock sale transaction.
    #[xbrl(concept = "us-gaap:SaleOfStockNumberOfSharesIssuedInTransaction")]
    pub sale_of_stock_number_of_shares_issued: Option<f64>,

    /// Consideration received from stock sale transaction.
    #[xbrl(concept = "us-gaap:SaleOfStockConsiderationReceivedOnTransaction")]
    pub sale_of_stock_consideration_received_on_transaction: Option<f64>,

    /// Shares issued during period - new issues.
    #[xbrl(concept = "us-gaap:StockIssuedDuringPeriodSharesNewIssues")]
    pub stock_issued_during_period_shares_new_issues: Option<f64>,

    /// Shares issued during period for share-based compensation.
    #[xbrl(concept = "us-gaap:StockIssuedDuringPeriodSharesShareBasedCompensation")]
    pub stock_issued_during_period_shares_share_based_compensation: Option<f64>,

    /// Shares issued during period - conversion of convertible securities.
    #[xbrl(concept = "us-gaap:StockIssuedDuringPeriodSharesConversionOfConvertibleSecurities")]
    pub stock_issued_during_period_shares_conversion_of_convertible_securities: Option<f64>,

    /// Restricted stock awards issued (gross).
    #[xbrl(concept = "us-gaap:StockIssuedDuringPeriodSharesRestrictedStockAwardGross")]
    pub stock_issued_during_period_shares_restricted_stock_award_gross: Option<f64>,

    /// Other shares issued during period.
    #[xbrl(concept = "us-gaap:StockIssuedDuringPeriodSharesOther")]
    pub stock_issued_during_period_shares_other: Option<f64>,

    /// Value of new stock issues during period.
    #[xbrl(concept = "us-gaap:StockIssuedDuringPeriodValueNewIssues")]
    pub stock_issued_during_period_value_new_issues: Option<f64>,

    /// Value of stock issued during period from conversion of convertible securities.
    #[xbrl(concept = "us-gaap:StockIssuedDuringPeriodValueConversionOfConvertibleSecurities")]
    pub stock_issued_during_period_value_conversion_of_convertible_securities: Option<f64>,

    /// Value of share-based compensation forfeited during period.
    #[xbrl(concept = "us-gaap:StockIssuedDuringPeriodValueShareBasedCompensationForfeited")]
    pub stock_issued_during_period_value_share_based_compensation_forfeited: Option<f64>,

    /// Value of other stock issued during period.
    #[xbrl(concept = "us-gaap:StockIssuedDuringPeriodValueOther")]
    pub stock_issued_during_period_value_other: Option<f64>,

    /// General stock issued amount.
    #[xbrl(concept = "us-gaap:StockIssued1")]
    pub stock_issued: Option<f64>,

    /// Shares issued (alternative field).
    #[xbrl(concept = "us-gaap:SharesIssued")]
    pub shares_issued_alt: Option<f64>,

    /// Shares issued during period for services.
    #[xbrl(concept = "us-gaap:StockIssuedDuringPeriodSharesIssuedForServices")]
    pub stock_issued_during_period_shares_issued_for_services: Option<f64>,

    /// Shares forfeited during period (share-based compensation).
    #[xbrl(concept = "us-gaap:StockIssuedDuringPeriodSharesShareBasedCompensationForfeited")]
    pub stock_issued_during_period_shares_share_based_compensation_forfeited: Option<f64>,

    /// Value of stock issued during period for services.
    #[xbrl(concept = "us-gaap:StockIssuedDuringPeriodValueIssuedForServices")]
    pub stock_issued_during_period_value_issued_for_services: Option<f64>,

    /// Shares redeemed or called during period.
    #[xbrl(concept = "us-gaap:StockRedeemedOrCalledDuringPeriodShares")]
    pub stock_redeemed_or_called_during_period_shares: Option<f64>,

    /// Shares repurchased and retired during period.
    #[xbrl(concept = "us-gaap:StockRepurchasedAndRetiredDuringPeriodShares")]
    pub stock_repurchased_and_retired_during_period_shares: Option<f64>,

    /// Value of stock repurchased and retired during period.
    #[xbrl(concept = "us-gaap:StockRepurchasedAndRetiredDuringPeriodValue")]
    pub stock_repurchased_and_retired_during_period_value: Option<f64>,

    /// Amount converted in stock conversion.
    #[xbrl(concept = "us-gaap:ConversionOfStockAmountConverted1")]
    pub conversion_of_stock_amount_converted: Option<f64>,

    /// Adjustments to APIC for warrant issuance.
    #[xbrl(concept = "us-gaap:AdjustmentsToAdditionalPaidInCapitalWarrantIssued")]
    pub adjustments_to_apic_warrant_issued: Option<f64>,

    /// Description of stock sale transaction.
    #[xbrl(concept = "us-gaap:SaleOfStockDescriptionOfTransaction")]
    pub sale_of_stock_description: Option<String>,

    /// Percentage of ownership before stock sale transaction.
    #[xbrl(concept = "us-gaap:SaleOfStockPercentageOfOwnershipBeforeTransaction")]
    pub sale_of_stock_percentage_ownership_before: Option<f64>,

    /// Share price in business acquisition.
    #[xbrl(concept = "us-gaap:BusinessAcquisitionSharePrice")]
    pub business_acquisition_share_price: Option<f64>,
}

/// Represents information about temporary equity and warrants (common in SPACs).
#[derive(Debug, Clone, Default, Serialize, Deserialize, FromXbrl)]
#[serde(default)]
pub struct TemporaryEquityAndWarrants {
    /// Carrying amount of temporary equity attributable to parent.
    #[xbrl(concept = "us-gaap:TemporaryEquityCarryingAmountAttributableToParent")]
    pub temporary_equity_carrying_amount: Option<f64>,

    /// Accretion of temporary equity to redemption value.
    #[xbrl(concept = "us-gaap:TemporaryEquityAccretionToRedemptionValue")]
    pub temporary_equity_accretion_to_redemption_value: Option<f64>,

    /// Adjustment to temporary equity accretion to redemption value.
    #[xbrl(concept = "us-gaap:TemporaryEquityAccretionToRedemptionValueAdjustment")]
    pub temporary_equity_accretion_adjustment: Option<f64>,

    /// Number of temporary equity shares issued.
    #[xbrl(concept = "us-gaap:TemporaryEquitySharesIssued")]
    pub temporary_equity_shares_issued: Option<f64>,

    /// Number of temporary equity shares outstanding.
    #[xbrl(concept = "us-gaap:TemporaryEquitySharesOutstanding")]
    pub temporary_equity_shares_outstanding: Option<f64>,

    /// Exercise price of warrants or rights.
    #[xbrl(concept = "us-gaap:ClassOfWarrantOrRightExercisePriceOfWarrantsOrRights1")]
    pub warrant_exercise_price: Option<f64>,

    /// Number of securities called by warrants or rights.
    #[xbrl(concept = "us-gaap:ClassOfWarrantOrRightNumberOfSecuritiesCalledByWarrantsOrRights")]
    pub warrant_number_of_securities_called: Option<f64>,

    /// Terms of outstanding warrants and rights.
    #[xbrl(concept = "us-gaap:WarrantsAndRightsOutstandingTerm")]
    pub warrants_and_rights_outstanding_term: Option<String>,

    /// Redemption price per share for temporary equity.
    #[xbrl(concept = "us-gaap:TemporaryEquityRedemptionPricePerShare")]
    pub temporary_equity_redemption_price_per_share: Option<f64>,

    /// Par or stated value per share for temporary equity.
    #[xbrl(concept = "us-gaap:TemporaryEquityParOrStatedValuePerShare")]
    pub temporary_equity_par_or_stated_value_per_share: Option<f64>,

    /// Redemption price per share for preferred stock.
    #[xbrl(concept = "us-gaap:PreferredStockRedemptionPricePerShare")]
    pub preferred_stock_redemption_price_per_share: Option<f64>,

    /// Value of temporary equity stock issued during period (new issues).
    #[xbrl(concept = "us-gaap:TemporaryEquityStockIssuedDuringPeriodValueNewIssues")]
    pub temporary_equity_stock_issued_value_new_issues: Option<f64>,

    /// Number of securities called by each warrant or right.
    #[xbrl(concept = "us-gaap:ClassOfWarrantOrRightNumberOfSecuritiesCalledByEachWarrantOrRight")]
    pub warrant_securities_per_warrant: Option<f64>,

    /// Number of warrants or rights outstanding.
    #[xbrl(concept = "us-gaap:ClassOfWarrantOrRightOutstanding")]
    pub warrants_or_rights_outstanding: Option<f64>,

    /// Warrants and rights outstanding (alternative).
    #[xbrl(concept = "us-gaap:WarrantsAndRightsOutstanding")]
    pub warrants_and_rights_outstanding_count: Option<f64>,

    /// Measurement input for warrants and rights outstanding.
    #[xbrl(concept = "us-gaap:WarrantsAndRightsOutstandingMeasurementInput")]
    pub warrants_measurement_input: Option<f64>,

    /// Valuation technique for warrants and rights (text).
    #[xbrl(concept = "us-gaap:WarrantsAndRightsOutstandingValuationTechniqueExtensibleList")]
    pub warrants_valuation_technique: Option<String>,

    /// Temporary equity table text block (disclosure).
    #[xbrl(concept = "us-gaap:TemporaryEquityTableTextBlock")]
    pub temporary_equity_disclosure: Option<String>,
}

/// Represents acquisition and business combination information (relevant for SPACs).
#[derive(Debug, Clone, Default, Serialize, Deserialize, FromXbrl)]
#[serde(default)]
pub struct BusinessCombinations {
    /// Consideration transferred in asset acquisition.
    #[xbrl(concept = "us-gaap:AssetAcquisitionConsiderationTransferred")]
    pub asset_acquisition_consideration_transferred: Option<f64>,

    /// Percentage of voting interests acquired in business acquisition.
    #[xbrl(concept = "us-gaap:BusinessAcquisitionPercentageOfVotingInterestsAcquired")]
    pub business_acquisition_percentage_of_voting_interests: Option<f64>,

    /// Percentage of equity interest in step acquisition.
    #[xbrl(
        concept = "us-gaap:BusinessCombinationStepAcquisitionEquityInterestInAcquireePercentage"
    )]
    pub step_acquisition_equity_interest_percentage: Option<f64>,

    /// Fair value of equity issued in business combination.
    #[xbrl(concept = "us-gaap:EquityIssuedInBusinessCombinationFairValueDisclosure")]
    pub equity_issued_fair_value: Option<f64>,

    /// Supplemental deferred purchase price.
    #[xbrl(concept = "us-gaap:SupplementalDeferredPurchasePrice")]
    pub supplemental_deferred_purchase_price: Option<f64>,

    /// Segment allocation table for business combination (text block).
    #[xbrl(concept = "us-gaap:BusinessCombinationSegmentAllocationTableTextBlock")]
    pub business_combination_segment_allocation_table: Option<String>,
}

/// Represents debt instruments and conversion details.
#[derive(Debug, Clone, Default, Serialize, Deserialize, FromXbrl)]
#[serde(default)]
pub struct DebtDetails {
    /// Face amount of debt instrument.
    #[xbrl(concept = "us-gaap:DebtInstrumentFaceAmount")]
    pub debt_instrument_face_amount: Option<f64>,

    /// Carrying amount of debt instrument.
    #[xbrl(concept = "us-gaap:DebtInstrumentCarryingAmount")]
    pub debt_instrument_carrying_amount: Option<f64>,

    /// Stated interest rate percentage on debt instrument.
    #[xbrl(concept = "us-gaap:DebtInstrumentInterestRateStatedPercentage")]
    pub debt_instrument_interest_rate_stated_percentage: Option<f64>,

    /// Conversion price of convertible debt instrument.
    #[xbrl(concept = "us-gaap:DebtInstrumentConvertibleConversionPrice1")]
    pub debt_convertible_conversion_price: Option<f64>,

    /// Amount of converted instrument in debt conversion.
    #[xbrl(concept = "us-gaap:DebtConversionConvertedInstrumentAmount1")]
    pub debt_conversion_converted_amount: Option<f64>,

    /// Debt issuance costs incurred during noncash or partial noncash transaction.
    #[xbrl(concept = "us-gaap:DebtIssuanceCostsIncurredDuringNoncashOrPartialNoncashTransaction")]
    pub debt_issuance_costs_incurred_during_noncash_transaction: Option<f64>,

    /// Notes issued value.
    #[xbrl(concept = "us-gaap:NotesIssued1")]
    pub notes_issued: Option<f64>,

    /// Extensible enumeration for related party notes payable type.
    #[xbrl(concept = "us-gaap:NotesPayableCurrentRelatedPartyTypeExtensibleEnumeration")]
    pub notes_payable_related_party_type: Option<String>,
}

/// Represents cash management and FDIC insurance details.
#[derive(Debug, Clone, Default, Serialize, Deserialize, FromXbrl)]
#[serde(default)]
pub struct CashManagement {
    /// Amount of cash that is FDIC insured.
    #[xbrl(concept = "us-gaap:CashFDICInsuredAmount")]
    pub cash_fdic_insured_amount: Option<f64>,

    /// Federal Deposit Insurance Corporation premium expense.
    #[xbrl(concept = "us-gaap:FederalDepositInsuranceCorporationPremiumExpense")]
    pub fdic_premium_expense: Option<f64>,
}

/// Represents commitments and contingencies placeholder.
#[derive(Debug, Clone, Default, Serialize, Deserialize, FromXbrl)]
#[serde(default)]
pub struct CommitmentsAndContingencies {
    /// Commitments and contingencies placeholder value.
    #[xbrl(concept = "us-gaap:CommitmentsAndContingencies")]
    pub commitments_and_contingencies: Option<f64>,
}

/// Represents segment and related party information.
#[derive(Debug, Clone, Default, Serialize, Deserialize, FromXbrl)]
#[serde(default)]
pub struct SegmentAndRelatedParty {
    /// Number of operating segments.
    #[xbrl(concept = "us-gaap:NumberOfOperatingSegments")]
    pub number_of_operating_segments: Option<i32>,

    /// Number of reportable segments.
    #[xbrl(concept = "us-gaap:NumberOfReportableSegments")]
    pub number_of_reportable_segments: Option<i32>,

    /// Amount of related party transaction.
    #[xbrl(concept = "us-gaap:RelatedPartyTransactionAmountsOfTransaction")]
    pub related_party_transaction_amount: Option<f64>,

    /// Extensible enumeration for segment reporting CODM title.
    #[xbrl(
        concept = "us-gaap:SegmentReportingCodmIndividualTitleAndPositionOrGroupOrCommitteeNameExtensibleEnumeration"
    )]
    pub segment_codm_title: Option<String>,
}

/// Represents tax-related disclosures.
#[derive(Debug, Clone, Default, Serialize, Deserialize, FromXbrl)]
#[serde(default)]
pub struct TaxDetails {
    /// Unrecognized tax benefits.
    #[xbrl(concept = "us-gaap:UnrecognizedTaxBenefits")]
    pub unrecognized_tax_benefits: Option<f64>,

    /// Accrued interest and penalties on unrecognized tax benefits.
    #[xbrl(concept = "us-gaap:UnrecognizedTaxBenefitsIncomeTaxPenaltiesAndInterestAccrued")]
    pub unrecognized_tax_benefits_penalties_and_interest: Option<f64>,
}

/// Represents risk concentrations and accounting policies.
#[derive(Debug, Clone, Default, Serialize, Deserialize, FromXbrl)]
#[serde(default)]
pub struct RisksAndPolicies {
    /// Concentration risk related to credit risk.
    #[xbrl(concept = "us-gaap:ConcentrationRiskCreditRisk")]
    pub concentration_risk_credit: Option<String>,

    /// Use of estimates policy narrative.
    #[xbrl(concept = "us-gaap:UseOfEstimates")]
    pub use_of_estimates: Option<String>,

    /// Dilutive securities disclosure.
    #[xbrl(concept = "us-gaap:DilutiveSecurities")]
    pub dilutive_securities: Option<String>,
}

/// Represents comprehensive income components.
#[derive(Debug, Clone, Default, Serialize, Deserialize, FromXbrl)]
#[serde(default)]
pub struct ComprehensiveIncomeDetails {
    /// Other comprehensive income from unrealized gains/losses on securities.
    #[xbrl(
        concept = "us-gaap:OtherComprehensiveIncomeUnrealizedHoldingGainLossOnSecuritiesArisingDuringPeriodBeforeTax"
    )]
    pub other_comprehensive_income_unrealized_securities: Option<f64>,

    /// Other comprehensive income from foreign currency transactions and translations.
    #[xbrl(
        concept = "us-gaap:OtherComprehensiveIncomeLossForeignCurrencyTransactionAndTranslationReclassificationAdjustmentFromAOCIRealizedUponSaleOrLiquidationBeforeTax"
    )]
    pub other_comprehensive_income_foreign_currency: Option<f64>,

    /// Other comprehensive income reclassification for held-to-maturity transfers.
    #[xbrl(
        concept = "us-gaap:OtherComprehensiveIncomeReclassificationAdjustmentForHeldToMaturityTransferredToAvailableForSaleSecuritiesBeforeTax"
    )]
    pub other_comprehensive_income_htm_reclassification: Option<f64>,

    /// Adjustments to additional paid-in capital for stock issuance costs.
    #[xbrl(concept = "us-gaap:AdjustmentsToAdditionalPaidInCapitalStockIssuedIssuanceCosts")]
    pub adjustments_to_paid_in_capital_issuance_costs: Option<f64>,
}

/// Represents key narrative text blocks for LLM analysis.
///
/// A filing's text blocks are its notes in full — tens to hundreds of
/// kilobytes of text. A caller that stores [`Financials`] and has no use for
/// them should clear this before it does.
#[derive(Debug, Clone, Default, Serialize, Deserialize, FromXbrl)]
#[serde(default)]
pub struct Narratives {
    /// Description of the nature of the entity's operations.
    #[xbrl(concept = "us-gaap:NatureOfOperations")]
    pub nature_of_operations: Option<String>,

    /// Disclosure text block for commitments and contingencies.
    #[xbrl(concept = "us-gaap:CommitmentsAndContingenciesDisclosureTextBlock")]
    pub commitments_and_contingencies: Option<String>,

    /// Text block describing significant accounting policies.
    #[xbrl(concept = "us-gaap:SignificantAccountingPoliciesTextBlock")]
    pub significant_accounting_policies: Option<String>,

    /// Text block describing consolidation policy.
    #[xbrl(concept = "us-gaap:ConsolidationPolicyTextBlock")]
    pub consolidation_policy: Option<String>,

    /// Combined basis of presentation and significant accounting policies text block.
    #[xbrl(concept = "us-gaap:BasisOfPresentationAndSignificantAccountingPoliciesTextBlock")]
    pub basis_of_presentation_and_significant_accounting_policies: Option<String>,

    /// Text block describing subsequent events.
    #[xbrl(concept = "us-gaap:SubsequentEventsTextBlock")]
    pub subsequent_events: Option<String>,

    /// Text block describing cash and cash equivalents policy.
    #[xbrl(concept = "us-gaap:CashAndCashEquivalentsPolicyTextBlock")]
    pub cash_and_cash_equivalents_policy: Option<String>,

    /// Text block describing earnings per share policy.
    #[xbrl(concept = "us-gaap:EarningsPerSharePolicyTextBlock")]
    pub earnings_per_share_policy: Option<String>,

    /// Policy for fair value of financial instruments.
    #[xbrl(concept = "us-gaap:FairValueOfFinancialInstrumentsPolicy")]
    pub fair_value_of_financial_instruments_policy: Option<String>,

    /// Text block describing income tax policy.
    #[xbrl(concept = "us-gaap:IncomeTaxPolicyTextBlock")]
    pub income_tax_policy: Option<String>,

    /// Text block describing derivatives policy.
    #[xbrl(concept = "us-gaap:DerivativesPolicyTextBlock")]
    pub derivatives_policy: Option<String>,

    /// Text block describing investment policy.
    #[xbrl(concept = "us-gaap:InvestmentPolicyTextBlock")]
    pub investment_policy: Option<String>,

    /// Text block describing new accounting pronouncements policy.
    #[xbrl(concept = "us-gaap:NewAccountingPronouncementsPolicyPolicyTextBlock")]
    pub new_accounting_pronouncements_policy: Option<String>,

    /// Text block disclosing related party transactions.
    #[xbrl(concept = "us-gaap:RelatedPartyTransactionsDisclosureTextBlock")]
    pub related_party_transactions: Option<String>,

    /// Text block for stockholders' equity note disclosure.
    #[xbrl(concept = "us-gaap:StockholdersEquityNoteDisclosureTextBlock")]
    pub stockholders_equity_note: Option<String>,

    /// Text block for fair value assets measured on recurring basis.
    #[xbrl(concept = "us-gaap:FairValueAssetsMeasuredOnRecurringBasisTextBlock")]
    pub fair_value_assets_measured_recurring: Option<String>,

    /// Basis of accounting policy text block.
    #[xbrl(concept = "us-gaap:BasisOfAccountingPolicyPolicyTextBlock")]
    pub basis_of_accounting_policy: Option<String>,

    /// Deferred charges policy text block.
    #[xbrl(concept = "us-gaap:DeferredChargesPolicyTextBlock")]
    pub deferred_charges_policy: Option<String>,

    /// Fair value assets and liabilities valuation techniques table text block.
    #[xbrl(
        concept = "us-gaap:FairValueAssetsAndLiabilitiesMeasuredOnRecurringAndNonrecurringBasisValuationTechniquesTableTextBlock"
    )]
    pub fair_value_valuation_techniques_table: Option<String>,

    /// Fair value disclosures text block.
    #[xbrl(concept = "us-gaap:FairValueDisclosuresTextBlock")]
    pub fair_value_disclosures: Option<String>,

    /// Fair value measurement policy text block.
    #[xbrl(concept = "us-gaap:FairValueMeasurementPolicyPolicyTextBlock")]
    pub fair_value_measurement_policy: Option<String>,

    /// Marketable securities policy text block.
    #[xbrl(concept = "us-gaap:MarketableSecuritiesPolicy")]
    pub marketable_securities_policy: Option<String>,

    /// Organization, consolidation, and presentation of financial statements disclosure text block.
    #[xbrl(
        concept = "us-gaap:OrganizationConsolidationAndPresentationOfFinancialStatementsDisclosureTextBlock"
    )]
    pub organization_consolidation_presentation_disclosure: Option<String>,

    /// Reconciliation of assets from segment to consolidated text block.
    #[xbrl(concept = "us-gaap:ReconciliationOfAssetsFromSegmentToConsolidatedTextBlock")]
    pub reconciliation_assets_segment_to_consolidated: Option<String>,

    /// Schedule of segment reporting information by segment text block.
    #[xbrl(concept = "us-gaap:ScheduleOfSegmentReportingInformationBySegmentTextBlock")]
    pub schedule_segment_reporting_information: Option<String>,

    /// Description of how CODM profit/loss measure is used.
    #[xbrl(concept = "us-gaap:SegmentReportingCodmProfitLossMeasureHowUsedDescription")]
    pub segment_codm_profit_loss_measure_description: Option<String>,

    /// Share-based compensation option and incentive plans policy text block.
    #[xbrl(concept = "us-gaap:ShareBasedCompensationOptionAndIncentivePlansPolicy")]
    pub share_based_compensation_policy: Option<String>,

    /// Text block for schedule of earnings per share basic and diluted.
    #[xbrl(concept = "us-gaap:ScheduleOfEarningsPerShareBasicAndDilutedTableTextBlock")]
    pub schedule_of_earnings_per_share: Option<String>,

    /// Text block for segment reporting disclosure.
    #[xbrl(concept = "us-gaap:SegmentReportingDisclosureTextBlock")]
    pub segment_reporting: Option<String>,

    /// Text block describing subsidiary of limited liability company or limited partnership.
    #[xbrl(
        concept = "us-gaap:ScheduleOfSubsidiaryOfLimitedLiabilityCompanyOrLimitedPartnershipDescriptionTextBlock"
    )]
    pub subsidiary_description: Option<String>,

    /// Text block for shares subject to mandatory redemption policy.
    #[xbrl(
        concept = "us-gaap:SharesSubjectToMandatoryRedemptionChangesInRedemptionValuePolicyTextBlock"
    )]
    pub shares_subject_to_mandatory_redemption_policy: Option<String>,
}

/// A composite structure holding all extracted US-GAAP financial data.
///
/// The three statements appear twice. `balance_sheet`, `income_statement` and
/// `cash_flow_statement` take, field by field, the fact that best matches the
/// period the filing reports on — in a 10-Q that is the year to date — and
/// fall back to a comparative or a dimensional breakdown when that is all the
/// filing tags. `balance_sheets`, `income_statements` and
/// `cash_flow_statements` hold one statement per period the filing reports,
/// each read from that period alone: the quarter beside the year to date, this
/// year beside last.
#[derive(Debug, Clone, Default, Serialize, Deserialize, FromXbrl)]
#[serde(default)]
pub struct Financials {
    /// Balance sheet information including assets, liabilities, and equity.
    #[xbrl(nested)]
    pub balance_sheet: BalanceSheet,

    /// Income statement information including revenues, expenses, and earnings.
    #[xbrl(nested)]
    pub income_statement: IncomeStatement,

    /// Cash flow statement information.
    #[xbrl(nested)]
    pub cash_flow_statement: CashFlowStatement,

    /// Detailed equity and stock information.
    #[xbrl(nested)]
    pub equity_details: EquityDetails,

    /// Temporary equity and warrant information (SPAC-specific).
    #[xbrl(nested)]
    pub temporary_equity_and_warrants: TemporaryEquityAndWarrants,

    /// Business combination and acquisition information.
    #[xbrl(nested)]
    pub business_combinations: BusinessCombinations,

    /// Comprehensive income details.
    #[xbrl(nested)]
    pub comprehensive_income_details: ComprehensiveIncomeDetails,

    /// Narrative disclosures for LLM analysis.
    #[xbrl(nested)]
    pub narratives: Narratives,

    /// The balance sheet at each date the filing reports one for, latest first.
    #[xbrl(each_period)]
    pub balance_sheets: Vec<BalanceSheet>,

    /// The income statement for each period the filing reports, latest first.
    /// A 10-Q carries the quarter and the year to date, each with its
    /// comparative from the year before.
    #[xbrl(each_period)]
    pub income_statements: Vec<IncomeStatement>,

    /// The cash flow statement for each period the filing reports, latest first.
    #[xbrl(each_period)]
    pub cash_flow_statements: Vec<CashFlowStatement>,
}

/// Extracts a comprehensive set of financial data from an XBRL document.
///
/// Fails on the first value that does not convert to its field's type; use
/// [`XbrlDataContext::extract_lenient`] to keep the rest of the struct.
pub fn extract_financials(context: &XbrlDataContext) -> Result<Financials> {
    context.extract()
}
