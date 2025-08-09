//! # DEI (Document and Entity Information) Taxonomy Extractor
//!
//! Provides high-level functions and structs for extracting standard document and entity
//! metadata from XBRL documents based on the SEC's DEI taxonomy.
//!
//! This module handles the comprehensive Document and Entity Information (DEI) taxonomy
//! as defined by the SEC. It extracts metadata about the filing document itself,
//! the reporting entity, contact information, audit details, and various regulatory flags.

use crate::error::Result;
use crate::serde_xbrl::{XbrlDataContext, from_data};
use serde::{Deserialize, Serialize};

/// Contains information about the document itself, such as its type, period, and fiscal details.
/// This data is primarily extracted from the `dei:` namespace in an XBRL filing, based on
/// the official SEC DEI taxonomy.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DocumentInfo {
    /// The type of the document, e.g., "10-K", "10-Q", "8-K".
    #[serde(rename(deserialize = "dei:DocumentType"), default)]
    pub document_type: Option<String>,

    /// The start date of the reporting period.
    #[serde(rename(deserialize = "dei:DocumentPeriodStartDate"), default)]
    pub document_period_start_date: Option<String>,

    /// The end date of the reporting period.
    #[serde(rename(deserialize = "dei:DocumentPeriodEndDate"), default)]
    pub document_period_end_date: Option<String>,

    /// The fiscal year focus of the document.
    #[serde(rename(deserialize = "dei:DocumentFiscalYearFocus"), default)]
    pub document_fiscal_year_focus: Option<String>,

    /// The fiscal period focus of the document (e.g., "Q1", "FY").
    #[serde(rename(deserialize = "dei:DocumentFiscalPeriodFocus"), default)]
    pub document_fiscal_period_focus: Option<String>,

    /// Flag indicating if this is an annual report.
    #[serde(rename(deserialize = "dei:DocumentAnnualReport"), default)]
    pub document_annual_report: Option<bool>,

    /// Flag indicating if this is a quarterly report.
    #[serde(rename(deserialize = "dei:DocumentQuarterlyReport"), default)]
    pub document_quarterly_report: Option<bool>,

    /// Flag indicating if this is a transition report.
    #[serde(rename(deserialize = "dei:DocumentTransitionReport"), default)]
    pub document_transition_report: Option<bool>,

    /// Flag indicating if the filing is a shell company report.
    #[serde(rename(deserialize = "dei:DocumentShellCompanyReport"), default)]
    pub document_shell_company_report: Option<bool>,

    /// The date on which shell company event occurred.
    #[serde(rename(deserialize = "dei:DocumentShellCompanyEventDate"), default)]
    pub document_shell_company_event_date: Option<String>,

    /// Flag indicating if the document contains a correction of an error in previously issued financial statements.
    #[serde(
        rename(deserialize = "dei:DocumentFinStmtErrorCorrectionFlag"),
        default
    )]
    pub document_fin_stmt_error_correction_flag: Option<bool>,

    /// Flag indicating if the document contains a restatement recovery analysis.
    #[serde(
        rename(deserialize = "dei:DocumentFinStmtRestatementRecoveryAnalysisFlag"),
        default
    )]
    pub document_fin_stmt_restatement_recovery_analysis_flag: Option<bool>,

    /// The accounting standard used (e.g., "US-GAAP", "IFRS").
    #[serde(rename(deserialize = "dei:DocumentAccountingStandard"), default)]
    pub document_accounting_standard: Option<String>,

    /// Other reporting standard item number.
    #[serde(rename(deserialize = "dei:OtherReportingStandardItemNumber"), default)]
    pub other_reporting_standard_item_number: Option<String>,

    /// Flag indicating if the filing is an amendment to a previous filing.
    #[serde(rename(deserialize = "dei:AmendmentFlag"), default)]
    pub amendment_flag: Option<bool>,

    /// Description of changes contained within an amended document.
    #[serde(rename(deserialize = "dei:AmendmentDescription"), default)]
    pub amendment_description: Option<String>,

    /// The date on which the filing becomes effective.
    #[serde(rename(deserialize = "dei:DocumentEffectiveDate"), default)]
    pub document_effective_date: Option<String>,

    /// The creation date of the document.
    #[serde(rename(deserialize = "dei:DocumentCreationDate"), default)]
    pub document_creation_date: Option<String>,

    /// Document version.
    #[serde(rename(deserialize = "dei:DocumentVersion"), default)]
    pub document_version: Option<String>,

    /// Document name.
    #[serde(rename(deserialize = "dei:DocumentName"), default)]
    pub document_name: Option<String>,

    /// Document title.
    #[serde(rename(deserialize = "dei:DocumentTitle"), default)]
    pub document_title: Option<String>,

    /// Document subtitle.
    #[serde(rename(deserialize = "dei:DocumentSubtitle"), default)]
    pub document_subtitle: Option<String>,

    /// Document description.
    #[serde(rename(deserialize = "dei:DocumentDescription"), default)]
    pub document_description: Option<String>,

    /// Document synopsis.
    #[serde(rename(deserialize = "dei:DocumentSynopsis"), default)]
    pub document_synopsis: Option<String>,

    /// Copyright information for the document.
    #[serde(rename(deserialize = "dei:DocumentCopyrightInformation"), default)]
    pub document_copyright_information: Option<String>,

    /// Flag indicating if this is a registration statement.
    #[serde(rename(deserialize = "dei:DocumentRegistrationStatement"), default)]
    pub document_registration_statement: Option<bool>,

    /// Registration statement amendment number.
    #[serde(
        rename(deserialize = "dei:RegistrationStatementAmendmentNumber"),
        default
    )]
    pub registration_statement_amendment_number: Option<String>,

    /// Flag indicating if this is a pre-effective amendment.
    #[serde(rename(deserialize = "dei:PreEffectiveAmendment"), default)]
    pub pre_effective_amendment: Option<bool>,

    /// Pre-effective amendment number.
    #[serde(rename(deserialize = "dei:PreEffectiveAmendmentNumber"), default)]
    pub pre_effective_amendment_number: Option<String>,

    /// Flag indicating if this is a post-effective amendment.
    #[serde(rename(deserialize = "dei:PostEffectiveAmendment"), default)]
    pub post_effective_amendment: Option<bool>,

    /// Post-effective amendment number.
    #[serde(rename(deserialize = "dei:PostEffectiveAmendmentNumber"), default)]
    pub post_effective_amendment_number: Option<String>,
}

/// Contains information about the reporting entity, such as its name, CIK, and filer status.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EntityInfo {
    /// The legal name of the reporting entity.
    #[serde(rename(deserialize = "dei:EntityRegistrantName"), default)]
    pub entity_registrant_name: Option<String>,

    /// Former legal or registered name of the entity.
    #[serde(
        rename(deserialize = "dei:EntityInformationFormerLegalOrRegisteredName"),
        default
    )]
    pub entity_information_former_legal_or_registered_name: Option<String>,

    /// Date to change former legal or registered name.
    #[serde(
        rename(deserialize = "dei:EntityInformationDateToChangeFormerLegalOrRegisteredName"),
        default
    )]
    pub entity_information_date_to_change_former_legal_or_registered_name: Option<String>,

    /// The Central Index Key (CIK) of the entity.
    #[serde(rename(deserialize = "dei:EntityCentralIndexKey"), default)]
    pub entity_central_index_key: Option<String>,

    /// The SEC file number for the entity.
    #[serde(rename(deserialize = "dei:EntityFileNumber"), default)]
    pub entity_file_number: Option<String>,

    /// Investment Company Act file number.
    #[serde(rename(deserialize = "dei:InvestmentCompanyActFileNumber"), default)]
    pub investment_company_act_file_number: Option<String>,

    /// The stock trading symbol.
    #[serde(rename(deserialize = "dei:TradingSymbol"), default)]
    pub trading_symbol: Option<String>,

    /// Flag indicating no trading symbol.
    #[serde(rename(deserialize = "dei:NoTradingSymbolFlag"), default)]
    pub no_trading_symbol_flag: Option<bool>,

    /// The name of the exchange on which the security is registered.
    #[serde(rename(deserialize = "dei:SecurityExchangeName"), default)]
    pub security_exchange_name: Option<String>,

    /// Title of 12(b) security.
    #[serde(rename(deserialize = "dei:Security12bTitle"), default)]
    pub security_12b_title: Option<String>,

    /// Title of 12(g) security.
    #[serde(rename(deserialize = "dei:Security12gTitle"), default)]
    pub security_12g_title: Option<String>,

    /// Security reporting obligation.
    #[serde(rename(deserialize = "dei:SecurityReportingObligation"), default)]
    pub security_reporting_obligation: Option<String>,

    /// The Legal Entity Identifier (LEI).
    #[serde(rename(deserialize = "dei:LegalEntityIdentifier"), default)]
    pub legal_entity_identifier: Option<String>,

    /// Entity legal form.
    #[serde(rename(deserialize = "dei:EntityLegalForm"), default)]
    pub entity_legal_form: Option<String>,

    /// Entity home country ISO code.
    #[serde(rename(deserialize = "dei:EntityHomeCountryIsoCode"), default)]
    pub entity_home_country_iso_code: Option<String>,

    /// Parent entity legal name.
    #[serde(rename(deserialize = "dei:ParentEntityLegalName"), default)]
    pub parent_entity_legal_name: Option<String>,

    /// Entity reporting currency ISO code.
    #[serde(rename(deserialize = "dei:EntityReportingCurrencyIsoCode"), default)]
    pub entity_reporting_currency_iso_code: Option<String>,

    /// Date of incorporation.
    #[serde(
        rename(deserialize = "dei:EntityIncorporationDateOfIncorporation"),
        default
    )]
    pub entity_incorporation_date_of_incorporation: Option<String>,

    /// The state or country code of incorporation.
    #[serde(
        rename(deserialize = "dei:EntityIncorporationStateCountryCode"),
        default
    )]
    pub entity_incorporation_state_country_code: Option<String>,

    /// The entity's Standard Industrial Classification (SIC) number.
    #[serde(rename(deserialize = "dei:EntityPrimarySicNumber"), default)]
    pub entity_primary_sic_number: Option<String>,

    /// The entity's Tax Identification Number.
    #[serde(rename(deserialize = "dei:EntityTaxIdentificationNumber"), default)]
    pub entity_tax_identification_number: Option<String>,

    /// Entity Data Universal Numbering System (DUNS) number.
    #[serde(
        rename(deserialize = "dei:EntityDataUniversalNumberingSystemNumber"),
        default
    )]
    pub entity_data_universal_numbering_system_number: Option<String>,

    /// The number of employees.
    #[serde(rename(deserialize = "dei:EntityNumberOfEmployees"), default)]
    pub entity_number_of_employees: Option<i64>,

    /// The entity's fiscal year end date, in MM-DD format.
    #[serde(rename(deserialize = "dei:CurrentFiscalYearEndDate"), default)]
    pub current_fiscal_year_end_date: Option<String>,

    /// Former fiscal year end date.
    #[serde(rename(deserialize = "dei:FormerFiscalYearEndDate"), default)]
    pub former_fiscal_year_end_date: Option<String>,

    /// The category of the filer (e.g., "Large Accelerated Filer").
    #[serde(rename(deserialize = "dei:EntityFilerCategory"), default)]
    pub entity_filer_category: Option<String>,

    /// Flag indicating if the entity is a well-known seasoned issuer.
    #[serde(rename(deserialize = "dei:EntityWellKnownSeasonedIssuer"), default)]
    pub entity_well_known_seasoned_issuer: Option<bool>,

    /// Flag indicating if the entity is a voluntary filer.
    #[serde(rename(deserialize = "dei:EntityVoluntaryFilers"), default)]
    pub entity_voluntary_filers: Option<bool>,

    /// Current reporting status of the entity.
    #[serde(rename(deserialize = "dei:EntityCurrentReportingStatus"), default)]
    pub entity_current_reporting_status: Option<bool>,

    /// Flag indicating if the entity is a shell company.
    #[serde(rename(deserialize = "dei:EntityShellCompany"), default)]
    pub entity_shell_company: Option<bool>,

    /// Flag indicating if the entity is a small business.
    #[serde(rename(deserialize = "dei:EntitySmallBusiness"), default)]
    pub entity_small_business: Option<bool>,

    /// Flag indicating if the entity is an emerging growth company.
    #[serde(rename(deserialize = "dei:EntityEmergingGrowthCompany"), default)]
    pub entity_emerging_growth_company: Option<bool>,

    /// Flag indicating if the entity is ex-transition period.
    #[serde(rename(deserialize = "dei:EntityExTransitionPeriod"), default)]
    pub entity_ex_transition_period: Option<bool>,

    /// The market value of the securities held by non-affiliates.
    #[serde(rename(deserialize = "dei:EntityPublicFloat"), default)]
    pub entity_public_float: Option<f64>,

    /// The number of common stock shares outstanding.
    #[serde(
        rename(deserialize = "dei:EntityCommonStockSharesOutstanding"),
        default
    )]
    pub entity_common_stock_shares_outstanding: Option<f64>,

    /// Par value per share for entity listing.
    #[serde(rename(deserialize = "dei:EntityListingParValuePerShare"), default)]
    pub entity_listing_par_value_per_share: Option<f64>,

    /// Flag indicating if entity listing is primary.
    #[serde(rename(deserialize = "dei:EntityListingPrimary"), default)]
    pub entity_listing_primary: Option<bool>,

    /// Flag indicating if entity listing is foreign.
    #[serde(rename(deserialize = "dei:EntityListingForeign"), default)]
    pub entity_listing_foreign: Option<bool>,

    /// Depositary receipt ratio for entity listing.
    #[serde(
        rename(deserialize = "dei:EntityListingDepositaryReceiptRatio"),
        default
    )]
    pub entity_listing_depositary_receipt_ratio: Option<f64>,

    /// Description of entity listing.
    #[serde(rename(deserialize = "dei:EntityListingDescription"), default)]
    pub entity_listing_description: Option<String>,

    /// Security trading currency for entity listing.
    #[serde(
        rename(deserialize = "dei:EntityListingSecurityTradingCurrency"),
        default
    )]
    pub entity_listing_security_trading_currency: Option<String>,

    /// Investment company type.
    #[serde(rename(deserialize = "dei:EntityInvCompanyType"), default)]
    pub entity_inv_company_type: Option<String>,

    /// Entity accounting standard.
    #[serde(rename(deserialize = "dei:EntityAccountingStandard"), default)]
    pub entity_accounting_standard: Option<String>,

    /// Flag indicating current interactive data status.
    #[serde(rename(deserialize = "dei:EntityInteractiveDataCurrent"), default)]
    pub entity_interactive_data_current: Option<bool>,

    /// Flag indicating current bankruptcy proceedings reporting.
    #[serde(
        rename(deserialize = "dei:EntityBankruptcyProceedingsReportingCurrent"),
        default
    )]
    pub entity_bankruptcy_proceedings_reporting_current: Option<bool>,
}

/// Contains address information for the entity.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EntityAddressInfo {
    /// Address description.
    #[serde(rename(deserialize = "dei:EntityAddressAddressDescription"), default)]
    pub entity_address_address_description: Option<String>,

    /// The first line of the entity's address.
    #[serde(rename(deserialize = "dei:EntityAddressAddressLine1"), default)]
    pub entity_address_address_line1: Option<String>,

    /// The second line of the entity's address.
    #[serde(rename(deserialize = "dei:EntityAddressAddressLine2"), default)]
    pub entity_address_address_line2: Option<String>,

    /// The third line of the entity's address.
    #[serde(rename(deserialize = "dei:EntityAddressAddressLine3"), default)]
    pub entity_address_address_line3: Option<String>,

    /// The city or town of the entity's address.
    #[serde(rename(deserialize = "dei:EntityAddressCityOrTown"), default)]
    pub entity_address_city_or_town: Option<String>,

    /// The state or province of the entity's address.
    #[serde(rename(deserialize = "dei:EntityAddressStateOrProvince"), default)]
    pub entity_address_state_or_province: Option<String>,

    /// The country of the entity's address.
    #[serde(rename(deserialize = "dei:EntityAddressCountry"), default)]
    pub entity_address_country: Option<String>,

    /// The postal or zip code of the entity's address.
    #[serde(rename(deserialize = "dei:EntityAddressPostalZipCode"), default)]
    pub entity_address_postal_zip_code: Option<String>,

    /// The region of the entity's address.
    #[serde(rename(deserialize = "dei:EntityAddressRegion"), default)]
    pub entity_address_region: Option<String>,
}

/// Contains contact information for the entity.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EntityContactInfo {
    /// Contact personnel name.
    #[serde(rename(deserialize = "dei:ContactPersonnelName"), default)]
    pub contact_personnel_name: Option<String>,

    /// Contact personnel email address.
    #[serde(rename(deserialize = "dei:ContactPersonnelEmailAddress"), default)]
    pub contact_personnel_email_address: Option<String>,

    /// Contact personnel fax number.
    #[serde(rename(deserialize = "dei:ContactPersonnelFaxNumber"), default)]
    pub contact_personnel_fax_number: Option<String>,

    /// Contact personnel URL.
    #[serde(
        rename(deserialize = "dei:ContactPersonnelUniformResourceLocatorUrl"),
        default
    )]
    pub contact_personnel_uniform_resource_locator_url: Option<String>,

    /// Phone/fax number description.
    #[serde(rename(deserialize = "dei:PhoneFaxNumberDescription"), default)]
    pub phone_fax_number_description: Option<String>,

    /// Country region.
    #[serde(rename(deserialize = "dei:CountryRegion"), default)]
    pub country_region: Option<String>,

    /// The area code for the entity's phone number.
    #[serde(rename(deserialize = "dei:CityAreaCode"), default)]
    pub city_area_code: Option<String>,

    /// The entity's local phone number.
    #[serde(rename(deserialize = "dei:LocalPhoneNumber"), default)]
    pub local_phone_number: Option<String>,

    /// Phone extension.
    #[serde(rename(deserialize = "dei:Extension"), default)]
    pub extension: Option<String>,
}

/// Contains information about the auditor of the filing.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuditInfo {
    /// The name of the auditor or audit firm.
    #[serde(rename(deserialize = "dei:AuditorName"), default)]
    pub auditor_name: Option<String>,

    /// The PCAOB-assigned number of the audit firm.
    #[serde(rename(deserialize = "dei:AuditorFirmId"), default)]
    pub auditor_firm_id: Option<String>,

    /// The location of the auditor.
    #[serde(rename(deserialize = "dei:AuditorLocation"), default)]
    pub auditor_location: Option<String>,

    /// Flag indicating if an auditor attestation report on internal control over financial reporting (ICFR) is included.
    #[serde(rename(deserialize = "dei:IcfrAuditorAttestationFlag"), default)]
    pub icfr_auditor_attestation_flag: Option<bool>,

    /// Flag indicating annual information form.
    #[serde(rename(deserialize = "dei:AnnualInformationForm"), default)]
    pub annual_information_form: Option<bool>,

    /// Flag indicating audited annual financial statements.
    #[serde(rename(deserialize = "dei:AuditedAnnualFinancialStatements"), default)]
    pub audited_annual_financial_statements: Option<bool>,
}

/// A composite structure holding all extracted Document and Entity Information (DEI).
/// This structure avoids `flatten` to maintain compatibility with Apache Arrow conversion.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeiInfo {
    /// Document-specific information.
    pub document: DocumentInfo,

    /// Entity-specific information.
    pub entity: EntityInfo,

    /// Entity address information.
    pub entity_address: EntityAddressInfo,

    /// Entity contact information.
    pub entity_contact: EntityContactInfo,

    /// Audit-specific information.
    pub audit: AuditInfo,
}

/// Extracts DEI (Document and Entity Information) from an XBRL document.
///
/// This function uses the high-performance `serde`-based deserializer to map
/// XBRL concepts directly to the `DeiInfo` struct.
pub fn extract_dei(context: &XbrlDataContext) -> Result<DeiInfo> {
    from_data(context)
}
