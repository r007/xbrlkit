//! # DEI (Document and Entity Information) Taxonomy Extractor
//!
//! Provides high-level functions and structs for extracting standard document and entity
//! metadata from XBRL documents based on the SEC's DEI taxonomy.
//!
//! This module handles the comprehensive Document and Entity Information (DEI) taxonomy
//! as defined by the SEC. It extracts metadata about the filing document itself,
//! the reporting entity, contact information, audit details, and various regulatory flags.

use crate::FromXbrl;
use crate::bind::{Fact, XbrlDataContext};
use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Contains information about the document itself, such as its type, period, and fiscal details.
/// This data is primarily extracted from the `dei:` namespace in an XBRL filing, based on
/// the official SEC DEI taxonomy.
#[derive(Debug, Clone, Default, Serialize, Deserialize, FromXbrl)]
#[serde(default)]
pub struct DocumentInfo {
    /// The type of the document, e.g., "10-K", "10-Q", "8-K".
    #[xbrl(concept = "dei:DocumentType")]
    pub document_type: Option<String>,

    /// The start date of the reporting period.
    #[xbrl(concept = "dei:DocumentPeriodStartDate")]
    pub document_period_start_date: Option<String>,

    /// The end date of the reporting period.
    #[xbrl(concept = "dei:DocumentPeriodEndDate")]
    pub document_period_end_date: Option<String>,

    /// The fiscal year focus of the document.
    #[xbrl(concept = "dei:DocumentFiscalYearFocus")]
    pub document_fiscal_year_focus: Option<String>,

    /// The fiscal period focus of the document (e.g., "Q1", "FY").
    #[xbrl(concept = "dei:DocumentFiscalPeriodFocus")]
    pub document_fiscal_period_focus: Option<String>,

    /// Flag indicating if this is an annual report.
    #[xbrl(concept = "dei:DocumentAnnualReport")]
    pub document_annual_report: Option<bool>,

    /// Flag indicating if this is a quarterly report.
    #[xbrl(concept = "dei:DocumentQuarterlyReport")]
    pub document_quarterly_report: Option<bool>,

    /// Flag indicating if this is a transition report.
    #[xbrl(concept = "dei:DocumentTransitionReport")]
    pub document_transition_report: Option<bool>,

    /// Flag indicating if the filing is a shell company report.
    #[xbrl(concept = "dei:DocumentShellCompanyReport")]
    pub document_shell_company_report: Option<bool>,

    /// The date on which shell company event occurred.
    #[xbrl(concept = "dei:DocumentShellCompanyEventDate")]
    pub document_shell_company_event_date: Option<String>,

    /// Flag indicating if the document contains a correction of an error in previously issued financial statements.
    #[xbrl(concept = "dei:DocumentFinStmtErrorCorrectionFlag")]
    pub document_fin_stmt_error_correction_flag: Option<bool>,

    /// Flag indicating if the document contains a restatement recovery analysis.
    #[xbrl(concept = "dei:DocumentFinStmtRestatementRecoveryAnalysisFlag")]
    pub document_fin_stmt_restatement_recovery_analysis_flag: Option<bool>,

    /// The accounting standard used (e.g., "US-GAAP", "IFRS").
    #[xbrl(concept = "dei:DocumentAccountingStandard")]
    pub document_accounting_standard: Option<String>,

    /// Other reporting standard item number.
    #[xbrl(concept = "dei:OtherReportingStandardItemNumber")]
    pub other_reporting_standard_item_number: Option<String>,

    /// Flag indicating if the filing is an amendment to a previous filing.
    #[xbrl(concept = "dei:AmendmentFlag")]
    pub amendment_flag: Option<bool>,

    /// Description of changes contained within an amended document.
    #[xbrl(concept = "dei:AmendmentDescription")]
    pub amendment_description: Option<String>,

    /// The date on which the filing becomes effective.
    #[xbrl(concept = "dei:DocumentEffectiveDate")]
    pub document_effective_date: Option<String>,

    /// The creation date of the document.
    #[xbrl(concept = "dei:DocumentCreationDate")]
    pub document_creation_date: Option<String>,

    /// Document version.
    #[xbrl(concept = "dei:DocumentVersion")]
    pub document_version: Option<String>,

    /// Document name.
    #[xbrl(concept = "dei:DocumentName")]
    pub document_name: Option<String>,

    /// Document title.
    #[xbrl(concept = "dei:DocumentTitle")]
    pub document_title: Option<String>,

    /// Document subtitle.
    #[xbrl(concept = "dei:DocumentSubtitle")]
    pub document_subtitle: Option<String>,

    /// Document description.
    #[xbrl(concept = "dei:DocumentDescription")]
    pub document_description: Option<String>,

    /// Document synopsis.
    #[xbrl(concept = "dei:DocumentSynopsis")]
    pub document_synopsis: Option<String>,

    /// Copyright information for the document.
    #[xbrl(concept = "dei:DocumentCopyrightInformation")]
    pub document_copyright_information: Option<String>,

    /// Flag indicating if this is a registration statement.
    #[xbrl(concept = "dei:DocumentRegistrationStatement")]
    pub document_registration_statement: Option<bool>,

    /// Registration statement amendment number.
    #[xbrl(concept = "dei:RegistrationStatementAmendmentNumber")]
    pub registration_statement_amendment_number: Option<String>,

    /// Flag indicating if this is a pre-effective amendment.
    #[xbrl(concept = "dei:PreEffectiveAmendment")]
    pub pre_effective_amendment: Option<bool>,

    /// Pre-effective amendment number.
    #[xbrl(concept = "dei:PreEffectiveAmendmentNumber")]
    pub pre_effective_amendment_number: Option<String>,

    /// Flag indicating if this is a post-effective amendment.
    #[xbrl(concept = "dei:PostEffectiveAmendment")]
    pub post_effective_amendment: Option<bool>,

    /// Post-effective amendment number.
    #[xbrl(concept = "dei:PostEffectiveAmendmentNumber")]
    pub post_effective_amendment_number: Option<String>,

    /// Flag indicating pre-commencement issuer tender offer.
    #[xbrl(concept = "dei:PreCommencementIssuerTenderOffer")]
    pub pre_commencement_issuer_tender_offer: Option<bool>,

    /// Flag indicating pre-commencement tender offer.
    #[xbrl(concept = "dei:PreCommencementTenderOffer")]
    pub pre_commencement_tender_offer: Option<bool>,

    /// Flag indicating soliciting material.
    #[xbrl(concept = "dei:SolicitingMaterial")]
    pub soliciting_material: Option<bool>,

    /// Flag indicating written communications.
    #[xbrl(concept = "dei:WrittenCommunications")]
    pub written_communications: Option<bool>,

    /// Text block for documents incorporated by reference.
    #[xbrl(concept = "dei:DocumentsIncorporatedByReferenceTextBlock")]
    pub documents_incorporated_by_reference_text_block: Option<String>,
}

/// Contains information about the reporting entity, such as its name, CIK, and filer status.
#[derive(Debug, Clone, Default, Serialize, Deserialize, FromXbrl)]
#[serde(default)]
pub struct EntityInfo {
    /// The legal name of the reporting entity.
    #[xbrl(concept = "dei:EntityRegistrantName")]
    pub entity_registrant_name: Option<String>,

    /// Former legal or registered name of the entity.
    #[xbrl(concept = "dei:EntityInformationFormerLegalOrRegisteredName")]
    pub entity_information_former_legal_or_registered_name: Option<String>,

    /// Date to change former legal or registered name.
    #[xbrl(concept = "dei:EntityInformationDateToChangeFormerLegalOrRegisteredName")]
    pub entity_information_date_to_change_former_legal_or_registered_name: Option<String>,

    /// The Central Index Key (CIK) of the entity.
    #[xbrl(concept = "dei:EntityCentralIndexKey")]
    pub entity_central_index_key: Option<String>,

    /// The SEC file number for the entity.
    #[xbrl(concept = "dei:EntityFileNumber")]
    pub entity_file_number: Option<String>,

    /// Investment Company Act file number.
    #[xbrl(concept = "dei:InvestmentCompanyActFileNumber")]
    pub investment_company_act_file_number: Option<String>,

    /// The stock trading symbol.
    #[xbrl(concept = "dei:TradingSymbol")]
    pub trading_symbol: Option<String>,

    /// Flag indicating no trading symbol.
    #[xbrl(concept = "dei:NoTradingSymbolFlag")]
    pub no_trading_symbol_flag: Option<bool>,

    /// The name of the exchange on which the security is registered.
    #[xbrl(concept = "dei:SecurityExchangeName")]
    pub security_exchange_name: Option<String>,

    /// Title of 12(b) security.
    #[xbrl(concept = "dei:Security12bTitle")]
    pub security_12b_title: Option<String>,

    /// Title of 12(g) security.
    #[xbrl(concept = "dei:Security12gTitle")]
    pub security_12g_title: Option<String>,

    /// Security reporting obligation.
    #[xbrl(concept = "dei:SecurityReportingObligation")]
    pub security_reporting_obligation: Option<String>,

    /// The Legal Entity Identifier (LEI).
    #[xbrl(concept = "dei:LegalEntityIdentifier")]
    pub legal_entity_identifier: Option<String>,

    /// Entity legal form.
    #[xbrl(concept = "dei:EntityLegalForm")]
    pub entity_legal_form: Option<String>,

    /// Entity home country ISO code.
    #[xbrl(concept = "dei:EntityHomeCountryIsoCode")]
    pub entity_home_country_iso_code: Option<String>,

    /// Parent entity legal name.
    #[xbrl(concept = "dei:ParentEntityLegalName")]
    pub parent_entity_legal_name: Option<String>,

    /// Entity reporting currency ISO code.
    #[xbrl(concept = "dei:EntityReportingCurrencyIsoCode")]
    pub entity_reporting_currency_iso_code: Option<String>,

    /// Date of incorporation.
    #[xbrl(concept = "dei:EntityIncorporationDateOfIncorporation")]
    pub entity_incorporation_date_of_incorporation: Option<String>,

    /// The state or country code of incorporation.
    #[xbrl(concept = "dei:EntityIncorporationStateCountryCode")]
    pub entity_incorporation_state_country_code: Option<String>,

    /// The entity's Standard Industrial Classification (SIC) number.
    #[xbrl(concept = "dei:EntityPrimarySicNumber")]
    pub entity_primary_sic_number: Option<String>,

    /// The entity's Tax Identification Number.
    #[xbrl(concept = "dei:EntityTaxIdentificationNumber")]
    pub entity_tax_identification_number: Option<String>,

    /// Entity Data Universal Numbering System (DUNS) number.
    #[xbrl(concept = "dei:EntityDataUniversalNumberingSystemNumber")]
    pub entity_data_universal_numbering_system_number: Option<String>,

    /// The number of employees.
    #[xbrl(concept = "dei:EntityNumberOfEmployees")]
    pub entity_number_of_employees: Option<i64>,

    /// The entity's fiscal year end date, in MM-DD format.
    #[xbrl(concept = "dei:CurrentFiscalYearEndDate")]
    pub current_fiscal_year_end_date: Option<String>,

    /// Former fiscal year end date.
    #[xbrl(concept = "dei:FormerFiscalYearEndDate")]
    pub former_fiscal_year_end_date: Option<String>,

    /// The category of the filer (e.g., "Large Accelerated Filer").
    #[xbrl(concept = "dei:EntityFilerCategory")]
    pub entity_filer_category: Option<String>,

    /// Flag indicating if the entity is a well-known seasoned issuer.
    #[xbrl(concept = "dei:EntityWellKnownSeasonedIssuer")]
    pub entity_well_known_seasoned_issuer: Option<bool>,

    /// Flag indicating if the entity is a voluntary filer.
    #[xbrl(concept = "dei:EntityVoluntaryFilers")]
    pub entity_voluntary_filers: Option<bool>,

    /// Current reporting status of the entity.
    #[xbrl(concept = "dei:EntityCurrentReportingStatus")]
    pub entity_current_reporting_status: Option<bool>,

    /// Flag indicating if the entity is a shell company.
    #[xbrl(concept = "dei:EntityShellCompany")]
    pub entity_shell_company: Option<bool>,

    /// Flag indicating if the entity is a small business.
    #[xbrl(concept = "dei:EntitySmallBusiness")]
    pub entity_small_business: Option<bool>,

    /// Flag indicating if the entity is an emerging growth company.
    #[xbrl(concept = "dei:EntityEmergingGrowthCompany")]
    pub entity_emerging_growth_company: Option<bool>,

    /// Flag indicating if the entity is ex-transition period.
    #[xbrl(concept = "dei:EntityExTransitionPeriod")]
    pub entity_ex_transition_period: Option<bool>,

    /// The market value of the securities held by non-affiliates.
    #[xbrl(concept = "dei:EntityPublicFloat")]
    pub entity_public_float: Option<f64>,

    /// The number of common stock shares outstanding.
    #[xbrl(concept = "dei:EntityCommonStockSharesOutstanding")]
    pub entity_common_stock_shares_outstanding: Option<f64>,

    /// The shares outstanding of each class, as the cover page reports them.
    ///
    /// A company with more than one class tags the concept once per class,
    /// each against `us-gaap:StatementClassOfStockAxis`, and tags no total —
    /// so `entity_common_stock_shares_outstanding` above is one class's count,
    /// and which one is an accident of the filing. This is all of them, each
    /// with the class it belongs to and the date it is as of.
    #[xbrl(concept = "dei:EntityCommonStockSharesOutstanding")]
    pub common_stock_shares_outstanding_by_class: Vec<Fact<f64>>,

    /// Par value per share for entity listing.
    #[xbrl(concept = "dei:EntityListingParValuePerShare")]
    pub entity_listing_par_value_per_share: Option<f64>,

    /// Flag indicating if entity listing is primary.
    #[xbrl(concept = "dei:EntityListingPrimary")]
    pub entity_listing_primary: Option<bool>,

    /// Flag indicating if entity listing is foreign.
    #[xbrl(concept = "dei:EntityListingForeign")]
    pub entity_listing_foreign: Option<bool>,

    /// Depositary receipt ratio for entity listing.
    #[xbrl(concept = "dei:EntityListingDepositaryReceiptRatio")]
    pub entity_listing_depositary_receipt_ratio: Option<f64>,

    /// Description of entity listing.
    #[xbrl(concept = "dei:EntityListingDescription")]
    pub entity_listing_description: Option<String>,

    /// Security trading currency for entity listing.
    #[xbrl(concept = "dei:EntityListingSecurityTradingCurrency")]
    pub entity_listing_security_trading_currency: Option<String>,

    /// Investment company type.
    #[xbrl(concept = "dei:EntityInvCompanyType")]
    pub entity_inv_company_type: Option<String>,

    /// Entity accounting standard.
    #[xbrl(concept = "dei:EntityAccountingStandard")]
    pub entity_accounting_standard: Option<String>,

    /// Flag indicating current interactive data status.
    #[xbrl(concept = "dei:EntityInteractiveDataCurrent")]
    pub entity_interactive_data_current: Option<bool>,

    /// Flag indicating current bankruptcy proceedings reporting.
    #[xbrl(concept = "dei:EntityBankruptcyProceedingsReportingCurrent")]
    pub entity_bankruptcy_proceedings_reporting_current: Option<bool>,
}

/// Contains address information for the entity.
#[derive(Debug, Clone, Default, Serialize, Deserialize, FromXbrl)]
#[serde(default)]
pub struct EntityAddressInfo {
    /// Address description.
    #[xbrl(concept = "dei:EntityAddressAddressDescription")]
    pub entity_address_address_description: Option<String>,

    /// The first line of the entity's address.
    #[xbrl(concept = "dei:EntityAddressAddressLine1")]
    pub entity_address_address_line1: Option<String>,

    /// The second line of the entity's address.
    #[xbrl(concept = "dei:EntityAddressAddressLine2")]
    pub entity_address_address_line2: Option<String>,

    /// The third line of the entity's address.
    #[xbrl(concept = "dei:EntityAddressAddressLine3")]
    pub entity_address_address_line3: Option<String>,

    /// The city or town of the entity's address.
    #[xbrl(concept = "dei:EntityAddressCityOrTown")]
    pub entity_address_city_or_town: Option<String>,

    /// The state or province of the entity's address.
    #[xbrl(concept = "dei:EntityAddressStateOrProvince")]
    pub entity_address_state_or_province: Option<String>,

    /// The country of the entity's address.
    #[xbrl(concept = "dei:EntityAddressCountry")]
    pub entity_address_country: Option<String>,

    /// The postal or zip code of the entity's address.
    #[xbrl(concept = "dei:EntityAddressPostalZipCode")]
    pub entity_address_postal_zip_code: Option<String>,

    /// The region of the entity's address.
    #[xbrl(concept = "dei:EntityAddressRegion")]
    pub entity_address_region: Option<String>,
}

/// Contains contact information for the entity.
#[derive(Debug, Clone, Default, Serialize, Deserialize, FromXbrl)]
#[serde(default)]
pub struct EntityContactInfo {
    /// Contact personnel name.
    #[xbrl(concept = "dei:ContactPersonnelName")]
    pub contact_personnel_name: Option<String>,

    /// Contact personnel email address.
    #[xbrl(concept = "dei:ContactPersonnelEmailAddress")]
    pub contact_personnel_email_address: Option<String>,

    /// Contact personnel fax number.
    #[xbrl(concept = "dei:ContactPersonnelFaxNumber")]
    pub contact_personnel_fax_number: Option<String>,

    /// Contact personnel URL.
    #[xbrl(concept = "dei:ContactPersonnelUniformResourceLocatorUrl")]
    pub contact_personnel_uniform_resource_locator_url: Option<String>,

    /// Phone/fax number description.
    #[xbrl(concept = "dei:PhoneFaxNumberDescription")]
    pub phone_fax_number_description: Option<String>,

    /// Country region.
    #[xbrl(concept = "dei:CountryRegion")]
    pub country_region: Option<String>,

    /// The area code for the entity's phone number.
    #[xbrl(concept = "dei:CityAreaCode")]
    pub city_area_code: Option<String>,

    /// The entity's local phone number.
    #[xbrl(concept = "dei:LocalPhoneNumber")]
    pub local_phone_number: Option<String>,

    /// Phone extension.
    #[xbrl(concept = "dei:Extension")]
    pub extension: Option<String>,
}

/// Contains information about the auditor of the filing.
#[derive(Debug, Clone, Default, Serialize, Deserialize, FromXbrl)]
#[serde(default)]
pub struct AuditInfo {
    /// The name of the auditor or audit firm.
    #[xbrl(concept = "dei:AuditorName")]
    pub auditor_name: Option<String>,

    /// The PCAOB-assigned number of the audit firm.
    #[xbrl(concept = "dei:AuditorFirmId")]
    pub auditor_firm_id: Option<String>,

    /// The location of the auditor.
    #[xbrl(concept = "dei:AuditorLocation")]
    pub auditor_location: Option<String>,

    /// Flag indicating if an auditor attestation report on internal control over financial reporting (ICFR) is included.
    #[xbrl(concept = "dei:IcfrAuditorAttestationFlag")]
    pub icfr_auditor_attestation_flag: Option<bool>,

    /// Flag indicating annual information form.
    #[xbrl(concept = "dei:AnnualInformationForm")]
    pub annual_information_form: Option<bool>,

    /// Flag indicating audited annual financial statements.
    #[xbrl(concept = "dei:AuditedAnnualFinancialStatements")]
    pub audited_annual_financial_statements: Option<bool>,
}

/// A composite structure holding all extracted Document and Entity Information (DEI).
#[derive(Debug, Clone, Default, Serialize, Deserialize, FromXbrl)]
#[serde(default)]
pub struct DeiInfo {
    /// Document-specific information.
    #[xbrl(nested)]
    pub document: DocumentInfo,

    /// Entity-specific information.
    #[xbrl(nested)]
    pub entity: EntityInfo,

    /// Entity address information.
    #[xbrl(nested)]
    pub entity_address: EntityAddressInfo,

    /// Entity contact information.
    #[xbrl(nested)]
    pub entity_contact: EntityContactInfo,

    /// Audit-specific information.
    #[xbrl(nested)]
    pub audit: AuditInfo,
}

/// Extracts DEI (Document and Entity Information) from an XBRL document.
///
/// Fails on the first value that does not convert to its field's type; use
/// [`XbrlDataContext::extract_lenient`] to keep the rest of the struct.
pub fn extract_dei(context: &XbrlDataContext) -> Result<DeiInfo> {
    context.extract()
}
