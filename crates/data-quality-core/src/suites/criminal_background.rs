use indexmap::IndexMap;
use serde_json::json;

use crate::config::DqConfig;
use crate::expectations::{ExpectationConfig, ExpectationMeta, GeneratedFrom};
use crate::suites::{fmt_test_id, SuiteGenerator};

/// Baseline suite for national criminal background check records.
/// Covers the 68-field schema: identity, timestamps, offender, case, court,
/// financial penalties, offense, arrest, warrant, conviction/sentencing,
/// commitment, supervision, amended records, and sex offender registry.
pub struct CriminalBackgroundSuite;

// ── Internal helpers ──────────────────────────────────────────────────────────

fn not_null(n: usize, col: &str, cat: &str, suite: &str, pfx: &str) -> ExpectationConfig {
    let mut kwargs = IndexMap::new();
    kwargs.insert("column".to_string(), json!(col));
    ExpectationConfig {
        expectation_type: "expect_column_values_to_not_be_null".to_string(),
        kwargs,
        meta: ExpectationMeta {
            test_id: fmt_test_id(pfx, n),
            category: cat.to_string(),
            suite: suite.to_string(),
            contract_field: Some(col.to_string()),
            contract_name: None,
            generated_from: GeneratedFrom::Baseline,
        },
    }
}

fn regex_match(n: usize, col: &str, regex: &str, cat: &str, suite: &str, pfx: &str) -> ExpectationConfig {
    let mut kwargs = IndexMap::new();
    kwargs.insert("column".to_string(), json!(col));
    kwargs.insert("regex".to_string(), json!(regex));
    ExpectationConfig {
        expectation_type: "expect_column_values_to_match_regex".to_string(),
        kwargs,
        meta: ExpectationMeta {
            test_id: fmt_test_id(pfx, n),
            category: cat.to_string(),
            suite: suite.to_string(),
            contract_field: Some(col.to_string()),
            contract_name: None,
            generated_from: GeneratedFrom::Baseline,
        },
    }
}

fn in_set(n: usize, col: &str, values: serde_json::Value, cat: &str, suite: &str, pfx: &str) -> ExpectationConfig {
    let mut kwargs = IndexMap::new();
    kwargs.insert("column".to_string(), json!(col));
    kwargs.insert("value_set".to_string(), values);
    ExpectationConfig {
        expectation_type: "expect_column_values_to_be_in_set".to_string(),
        kwargs,
        meta: ExpectationMeta {
            test_id: fmt_test_id(pfx, n),
            category: cat.to_string(),
            suite: suite.to_string(),
            contract_field: Some(col.to_string()),
            contract_name: None,
            generated_from: GeneratedFrom::Baseline,
        },
    }
}

fn length_between(n: usize, col: &str, min: Option<u64>, max: u64, cat: &str, suite: &str, pfx: &str) -> ExpectationConfig {
    let mut kwargs = IndexMap::new();
    kwargs.insert("column".to_string(), json!(col));
    if let Some(mn) = min {
        kwargs.insert("min_value".to_string(), json!(mn));
    }
    kwargs.insert("max_value".to_string(), json!(max));
    ExpectationConfig {
        expectation_type: "expect_column_value_lengths_to_be_between".to_string(),
        kwargs,
        meta: ExpectationMeta {
            test_id: fmt_test_id(pfx, n),
            category: cat.to_string(),
            suite: suite.to_string(),
            contract_field: Some(col.to_string()),
            contract_name: None,
            generated_from: GeneratedFrom::Baseline,
        },
    }
}

fn unique(n: usize, col: &str, cat: &str, suite: &str, pfx: &str) -> ExpectationConfig {
    let mut kwargs = IndexMap::new();
    kwargs.insert("column".to_string(), json!(col));
    ExpectationConfig {
        expectation_type: "expect_column_values_to_be_unique".to_string(),
        kwargs,
        meta: ExpectationMeta {
            test_id: fmt_test_id(pfx, n),
            category: cat.to_string(),
            suite: suite.to_string(),
            contract_field: Some(col.to_string()),
            contract_name: None,
            generated_from: GeneratedFrom::Baseline,
        },
    }
}

// ── SuiteGenerator impl ───────────────────────────────────────────────────────

impl SuiteGenerator for CriminalBackgroundSuite {
    fn suite_name(&self) -> &str { "criminal_background_suite" }
    fn category(&self) -> &str { "criminal_record" }
    fn test_id_prefix(&self) -> &str { "CBC" }
    fn test_id_start(&self) -> usize { 1 }

    #[allow(clippy::cognitive_complexity)]
    fn generate(&self, _config: &DqConfig) -> Vec<ExpectationConfig> {
        let suite = self.suite_name();
        let pfx = self.test_id_prefix();
        let mut e: Vec<ExpectationConfig> = Vec::new();
        let mut n: usize = 1;

        // ── CBC001–CBC010: Schema-level tests ─────────────────────────────────

        // CBC001: All 68 column names present
        {
            let all_columns = json!([
                "id","hash","category","source","sourcestate","sourcename",
                "dateadded","offenderid","casenum","warrantnum","casetype",
                "court","courtdate","counts","plea","bail","bond","bond_type",
                "fine","courtcosts","restitution","offensedate","offensecode",
                "offensedesc1","offensedesc2","arrestdate","arrestagency",
                "arrestlocation","warrantdate","warrantcounty","warrantstate",
                "chargesfileddate","convictiondate","convictionplace",
                "sentenceyyymmddd","probationyyymmddd","sentencedate",
                "sentencesuspended","probationdate","disposition","dispositiondate",
                "commitmentlocation","commitmentcounty","commitmentstate",
                "commitmentdate","supervisiondate","supervisiontype",
                "supervisioncounty","supervisionstate","amendedcharge",
                "amendedcasetype","amendeddisposition","amendeddispositiondate",
                "fbinum","prisonerid","registrationdate","lastreporteddate",
                "profileupdatedate","sexoffendertier","victimminor",
                "victimrelationship","offenderage","offenderdob",
                "offenderfirstname","offenderlastname","offendermiddlename",
                "offenderrace","offendersex"
            ]);
            let mut kwargs = IndexMap::new();
            kwargs.insert("column_set".to_string(), all_columns);
            kwargs.insert("exact_match".to_string(), json!(true));
            e.push(ExpectationConfig {
                expectation_type: "expect_table_columns_to_match_set".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, n),
                    category: "criminal_record_schema".to_string(),
                    suite: suite.to_string(),
                    contract_field: None,
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
            n += 1;
        }

        // CBC002: Column count = 68
        {
            let mut kwargs = IndexMap::new();
            kwargs.insert("value".to_string(), json!(68));
            e.push(ExpectationConfig {
                expectation_type: "expect_table_column_count_to_equal".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, n),
                    category: "criminal_record_schema".to_string(),
                    suite: suite.to_string(),
                    contract_field: None,
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
            n += 1;
        }

        // CBC003: Row count >= 1
        {
            let mut kwargs = IndexMap::new();
            kwargs.insert("min_value".to_string(), json!(1));
            e.push(ExpectationConfig {
                expectation_type: "expect_table_row_count_to_be_between".to_string(),
                kwargs,
                meta: ExpectationMeta {
                    test_id: fmt_test_id(pfx, n),
                    category: "criminal_record_schema".to_string(),
                    suite: suite.to_string(),
                    contract_field: None,
                    contract_name: None,
                    generated_from: GeneratedFrom::Baseline,
                },
            });
            n += 1;
        }

        // CBC004: id uniqueness
        e.push(unique(n, "id", "criminal_record_identity", suite, pfx)); n += 1;

        // CBC005: hash not null
        e.push(not_null(n, "hash", "criminal_record_identity", suite, pfx)); n += 1;

        // CBC006: hash format — hex 16 chars
        e.push(regex_match(n, "hash", r"^[0-9a-f]{16}$", "criminal_record_identity", suite, pfx)); n += 1;

        // CBC007: category enum
        e.push(in_set(n, "category",
            json!(["CRIMINAL","FELONY","MISDEMEANOR","TRAFFIC","SEX_OFFENDER","CIVIL"]),
            "criminal_record_identity", suite, pfx)); n += 1;

        // CBC008: sourcestate 2-letter US state code
        e.push(regex_match(n, "sourcestate", r"^[A-Z]{2}$", "criminal_record_identity", suite, pfx)); n += 1;

        // CBC009: source max 100 chars
        e.push(length_between(n, "source", Some(1), 100, "criminal_record_identity", suite, pfx)); n += 1;

        // CBC010: source not null
        e.push(not_null(n, "source", "criminal_record_identity", suite, pfx)); n += 1;

        // ── CBC011–CBC020: Offender identity & case identifiers ───────────────

        // CBC011: offenderid not null
        e.push(not_null(n, "offenderid", "criminal_record_offender", suite, pfx)); n += 1;

        // CBC012: offenderid unique
        e.push(unique(n, "offenderid", "criminal_record_offender", suite, pfx)); n += 1;

        // CBC013: offenderid max 100 chars
        e.push(length_between(n, "offenderid", Some(1), 100, "criminal_record_offender", suite, pfx)); n += 1;

        // CBC014: casenum not null
        e.push(not_null(n, "casenum", "criminal_record_case", suite, pfx)); n += 1;

        // CBC015: casenum unique
        e.push(unique(n, "casenum", "criminal_record_case", suite, pfx)); n += 1;

        // CBC016: casenum max 200 chars
        e.push(length_between(n, "casenum", Some(1), 200, "criminal_record_case", suite, pfx)); n += 1;

        // CBC017: casetype not null
        e.push(not_null(n, "casetype", "criminal_record_case", suite, pfx)); n += 1;

        // CBC018: casetype enum
        e.push(in_set(n, "casetype",
            json!(["Felony","Misdemeanor","Traffic","Infraction","Civil","Other"]),
            "criminal_record_case", suite, pfx)); n += 1;

        // CBC019: fbinum not null
        e.push(not_null(n, "fbinum", "criminal_record_identity", suite, pfx)); n += 1;

        // CBC020: fbinum max 100 chars
        e.push(length_between(n, "fbinum", None, 100, "criminal_record_identity", suite, pfx)); n += 1;

        // ── CBC021–CBC050: Date format tests ──────────────────────────────────

        // CBC021: offensedate not null
        e.push(not_null(n, "offensedate", "criminal_record_offense", suite, pfx)); n += 1;
        // CBC022: offensedate format yyyymmdd
        e.push(regex_match(n, "offensedate", r"^\d{8}$", "criminal_record_offense", suite, pfx)); n += 1;

        // CBC023: arrestdate not null
        e.push(not_null(n, "arrestdate", "criminal_record_arrest", suite, pfx)); n += 1;
        // CBC024: arrestdate format
        e.push(regex_match(n, "arrestdate", r"^\d{8}$", "criminal_record_arrest", suite, pfx)); n += 1;

        // CBC025: convictiondate not null
        e.push(not_null(n, "convictiondate", "criminal_record_conviction", suite, pfx)); n += 1;
        // CBC026: convictiondate format
        e.push(regex_match(n, "convictiondate", r"^\d{8}$", "criminal_record_conviction", suite, pfx)); n += 1;

        // CBC027: sentencedate not null
        e.push(not_null(n, "sentencedate", "criminal_record_conviction", suite, pfx)); n += 1;
        // CBC028: sentencedate format
        e.push(regex_match(n, "sentencedate", r"^\d{8}$", "criminal_record_conviction", suite, pfx)); n += 1;

        // CBC029: dispositiondate not null
        e.push(not_null(n, "dispositiondate", "criminal_record_conviction", suite, pfx)); n += 1;
        // CBC030: dispositiondate format
        e.push(regex_match(n, "dispositiondate", r"^\d{8}$", "criminal_record_conviction", suite, pfx)); n += 1;

        // CBC031: courtdate not null
        e.push(not_null(n, "courtdate", "criminal_record_court", suite, pfx)); n += 1;
        // CBC032: courtdate format (semicolon-delimited yyyymmdd values)
        e.push(regex_match(n, "courtdate", r"^\d{8}(;\d{8})*$", "criminal_record_court", suite, pfx)); n += 1;

        // CBC033: warrantdate not null
        e.push(not_null(n, "warrantdate", "criminal_record_warrant", suite, pfx)); n += 1;
        // CBC034: warrantdate format
        e.push(regex_match(n, "warrantdate", r"^\d{8}$", "criminal_record_warrant", suite, pfx)); n += 1;

        // CBC035: chargesfileddate not null
        e.push(not_null(n, "chargesfileddate", "criminal_record_conviction", suite, pfx)); n += 1;
        // CBC036: chargesfileddate format
        e.push(regex_match(n, "chargesfileddate", r"^\d{8}$", "criminal_record_conviction", suite, pfx)); n += 1;

        // CBC037: commitmentdate not null
        e.push(not_null(n, "commitmentdate", "criminal_record_commitment", suite, pfx)); n += 1;
        // CBC038: commitmentdate format
        e.push(regex_match(n, "commitmentdate", r"^\d{8}$", "criminal_record_commitment", suite, pfx)); n += 1;

        // CBC039: supervisiondate not null
        e.push(not_null(n, "supervisiondate", "criminal_record_supervision", suite, pfx)); n += 1;
        // CBC040: supervisiondate format
        e.push(regex_match(n, "supervisiondate", r"^\d{8}$", "criminal_record_supervision", suite, pfx)); n += 1;

        // CBC041: probationdate not null
        e.push(not_null(n, "probationdate", "criminal_record_conviction", suite, pfx)); n += 1;
        // CBC042: probationdate format
        e.push(regex_match(n, "probationdate", r"^\d{8}$", "criminal_record_conviction", suite, pfx)); n += 1;

        // CBC043: registrationdate not null
        e.push(not_null(n, "registrationdate", "criminal_record_sex_offender", suite, pfx)); n += 1;
        // CBC044: registrationdate format
        e.push(regex_match(n, "registrationdate", r"^\d{8}$", "criminal_record_sex_offender", suite, pfx)); n += 1;

        // CBC045: lastreporteddate not null
        e.push(not_null(n, "lastreporteddate", "criminal_record_sex_offender", suite, pfx)); n += 1;
        // CBC046: lastreporteddate format
        e.push(regex_match(n, "lastreporteddate", r"^\d{8}$", "criminal_record_sex_offender", suite, pfx)); n += 1;

        // CBC047: profileupdatedate not null
        e.push(not_null(n, "profileupdatedate", "criminal_record_sex_offender", suite, pfx)); n += 1;
        // CBC048: profileupdatedate format
        e.push(regex_match(n, "profileupdatedate", r"^\d{8}$", "criminal_record_sex_offender", suite, pfx)); n += 1;

        // CBC049: dateadded not null
        e.push(not_null(n, "dateadded", "criminal_record_identity", suite, pfx)); n += 1;
        // CBC050: dateadded format (YYYY-MM-DD or yyyymmdd)
        e.push(regex_match(n, "dateadded", r"^(\d{4}-\d{2}-\d{2}|\d{8})$", "criminal_record_identity", suite, pfx)); n += 1;

        // ── CBC051–CBC060: Plea and disposition tests ─────────────────────────

        // CBC051: plea enum
        e.push(in_set(n, "plea",
            json!(["Guilty","Not Guilty","No Contest","Nolo Contendere","Alford Plea","Not Responsible"]),
            "criminal_record_court", suite, pfx)); n += 1;

        // CBC052: plea not null
        e.push(not_null(n, "plea", "criminal_record_court", suite, pfx)); n += 1;

        // CBC053: disposition not null
        e.push(not_null(n, "disposition", "criminal_record_conviction", suite, pfx)); n += 1;

        // CBC054: disposition max 400 chars
        e.push(length_between(n, "disposition", None, 400, "criminal_record_conviction", suite, pfx)); n += 1;

        // CBC055: sentencesuspended enum
        e.push(in_set(n, "sentencesuspended",
            json!(["Y","N","Yes","No","TRUE","FALSE"]),
            "criminal_record_conviction", suite, pfx)); n += 1;

        // CBC056: sentencesuspended not null
        e.push(not_null(n, "sentencesuspended", "criminal_record_conviction", suite, pfx)); n += 1;

        // CBC057: court not null
        e.push(not_null(n, "court", "criminal_record_court", suite, pfx)); n += 1;

        // CBC058: court max 200 chars
        e.push(length_between(n, "court", None, 200, "criminal_record_court", suite, pfx)); n += 1;

        // CBC059: counts not null
        e.push(not_null(n, "counts", "criminal_record_court", suite, pfx)); n += 1;

        // CBC060: amendeddispositiondate format
        e.push(regex_match(n, "amendeddispositiondate", r"^\d{8}$", "criminal_record_amended", suite, pfx)); n += 1;

        // ── CBC061–CBC080: Financial field tests ──────────────────────────────

        // CBC061: bail not null
        e.push(not_null(n, "bail", "criminal_record_financial", suite, pfx)); n += 1;
        // CBC062: bail monetary pattern
        e.push(regex_match(n, "bail", r"^\$?[\d,]+(\.\d{2})?$", "criminal_record_financial", suite, pfx)); n += 1;
        // CBC063: bail max 100 chars
        e.push(length_between(n, "bail", None, 100, "criminal_record_financial", suite, pfx)); n += 1;

        // CBC064: bond not null
        e.push(not_null(n, "bond", "criminal_record_financial", suite, pfx)); n += 1;
        // CBC065: bond monetary pattern
        e.push(regex_match(n, "bond", r"^\$?[\d,]+(\.\d{2})?$", "criminal_record_financial", suite, pfx)); n += 1;
        // CBC066: bond max 1000 chars
        e.push(length_between(n, "bond", None, 1000, "criminal_record_financial", suite, pfx)); n += 1;

        // CBC067: fine not null
        e.push(not_null(n, "fine", "criminal_record_financial", suite, pfx)); n += 1;
        // CBC068: fine monetary pattern
        e.push(regex_match(n, "fine", r"^\$?[\d,]+(\.\d{2})?$", "criminal_record_financial", suite, pfx)); n += 1;
        // CBC069: fine max 100 chars
        e.push(length_between(n, "fine", None, 100, "criminal_record_financial", suite, pfx)); n += 1;

        // CBC070: courtcosts not null
        e.push(not_null(n, "courtcosts", "criminal_record_financial", suite, pfx)); n += 1;
        // CBC071: courtcosts monetary pattern
        e.push(regex_match(n, "courtcosts", r"^\$?[\d,]+(\.\d{2})?$", "criminal_record_financial", suite, pfx)); n += 1;
        // CBC072: courtcosts max 100 chars
        e.push(length_between(n, "courtcosts", None, 100, "criminal_record_financial", suite, pfx)); n += 1;

        // CBC073: restitution not null
        e.push(not_null(n, "restitution", "criminal_record_financial", suite, pfx)); n += 1;
        // CBC074: restitution monetary pattern
        e.push(regex_match(n, "restitution", r"^\$?[\d,]+(\.\d{2})?$", "criminal_record_financial", suite, pfx)); n += 1;
        // CBC075: restitution max 100 chars
        e.push(length_between(n, "restitution", None, 100, "criminal_record_financial", suite, pfx)); n += 1;

        // CBC076: bond_type not null
        e.push(not_null(n, "bond_type", "criminal_record_financial", suite, pfx)); n += 1;
        // CBC077: bond_type max 200 chars
        e.push(length_between(n, "bond_type", None, 200, "criminal_record_financial", suite, pfx)); n += 1;

        // CBC078: sourcename not null
        e.push(not_null(n, "sourcename", "criminal_record_identity", suite, pfx)); n += 1;
        // CBC079: sourcename max 100 chars
        e.push(length_between(n, "sourcename", None, 100, "criminal_record_identity", suite, pfx)); n += 1;

        // CBC080: sourcestate not null
        e.push(not_null(n, "sourcestate", "criminal_record_identity", suite, pfx)); n += 1;

        // ── CBC081–CBC090: Warrant tests ──────────────────────────────────────

        // CBC081: warrantnum not null
        e.push(not_null(n, "warrantnum", "criminal_record_warrant", suite, pfx)); n += 1;

        // CBC082: warrantnum max 1000 chars
        e.push(length_between(n, "warrantnum", None, 1000, "criminal_record_warrant", suite, pfx)); n += 1;

        // CBC083: warrantstate not null
        e.push(not_null(n, "warrantstate", "criminal_record_warrant", suite, pfx)); n += 1;

        // CBC084: warrantstate 2-letter state code
        e.push(regex_match(n, "warrantstate", r"^[A-Z]{2}$", "criminal_record_warrant", suite, pfx)); n += 1;

        // CBC085: warrantcounty not null
        e.push(not_null(n, "warrantcounty", "criminal_record_warrant", suite, pfx)); n += 1;

        // CBC086: warrantdate not null (warrant section)
        e.push(not_null(n, "warrantdate", "criminal_record_warrant", suite, pfx)); n += 1;

        // CBC087: warrantdate format (yyyymmdd)
        e.push(regex_match(n, "warrantdate", r"^\d{8}$", "criminal_record_warrant", suite, pfx)); n += 1;

        // CBC088: warrantcounty max 100 chars
        e.push(length_between(n, "warrantcounty", None, 100, "criminal_record_warrant", suite, pfx)); n += 1;

        // CBC089: warrantstate max 100 chars
        e.push(length_between(n, "warrantstate", None, 100, "criminal_record_warrant", suite, pfx)); n += 1;

        // CBC090: category not null
        e.push(not_null(n, "category", "criminal_record_identity", suite, pfx)); n += 1;

        // ── CBC091–CBC110: Sex offender registry tests ────────────────────────

        // CBC091: registrationdate not null (sex offender section)
        e.push(not_null(n, "registrationdate", "criminal_record_sex_offender", suite, pfx)); n += 1;

        // CBC092: registrationdate format
        e.push(regex_match(n, "registrationdate", r"^\d{8}$", "criminal_record_sex_offender", suite, pfx)); n += 1;

        // CBC093: sexoffendertier not null
        e.push(not_null(n, "sexoffendertier", "criminal_record_sex_offender", suite, pfx)); n += 1;

        // CBC094: sexoffendertier enum
        e.push(in_set(n, "sexoffendertier",
            json!(["I","II","III","1","2","3","Tier I","Tier II","Tier III"]),
            "criminal_record_sex_offender", suite, pfx)); n += 1;

        // CBC095: victimminor enum
        e.push(in_set(n, "victimminor",
            json!(["Y","N","Yes","No","TRUE","FALSE"]),
            "criminal_record_sex_offender", suite, pfx)); n += 1;

        // CBC096: offenderdob not null
        e.push(not_null(n, "offenderdob", "criminal_record_sex_offender", suite, pfx)); n += 1;

        // CBC097: offenderdob format
        e.push(regex_match(n, "offenderdob", r"^\d{8}$", "criminal_record_sex_offender", suite, pfx)); n += 1;

        // CBC098: offenderfirstname not null
        e.push(not_null(n, "offenderfirstname", "criminal_record_sex_offender", suite, pfx)); n += 1;

        // CBC099: offenderlastname not null
        e.push(not_null(n, "offenderlastname", "criminal_record_sex_offender", suite, pfx)); n += 1;

        // CBC100: offendersex enum
        e.push(in_set(n, "offendersex",
            json!(["M","F","Male","Female","Non-Binary","Unknown"]),
            "criminal_record_sex_offender", suite, pfx)); n += 1;

        // ── CBC101–CBC115: Supervision and commitment tests ───────────────────

        // CBC101: supervisiontype not null
        e.push(not_null(n, "supervisiontype", "criminal_record_supervision", suite, pfx)); n += 1;

        // CBC102: supervisiontype enum
        e.push(in_set(n, "supervisiontype",
            json!(["Probation","Parole","Supervised Release","House Arrest","Electronic Monitoring"]),
            "criminal_record_supervision", suite, pfx)); n += 1;

        // CBC103: supervisionstate regex
        e.push(regex_match(n, "supervisionstate", r"^[A-Z]{2}$", "criminal_record_supervision", suite, pfx)); n += 1;

        // CBC104: commitmentstate not null
        e.push(not_null(n, "commitmentstate", "criminal_record_commitment", suite, pfx)); n += 1;

        // CBC105: commitmentstate regex
        e.push(regex_match(n, "commitmentstate", r"^[A-Z]{2}$", "criminal_record_commitment", suite, pfx)); n += 1;

        // CBC106: commitmentdate not null
        e.push(not_null(n, "commitmentdate", "criminal_record_commitment", suite, pfx)); n += 1;

        // CBC107: commitmentdate format
        e.push(regex_match(n, "commitmentdate", r"^\d{8}$", "criminal_record_commitment", suite, pfx)); n += 1;

        // CBC108: commitmentlocation not null
        e.push(not_null(n, "commitmentlocation", "criminal_record_commitment", suite, pfx)); n += 1;

        // CBC109: supervisiondate not null
        e.push(not_null(n, "supervisiondate", "criminal_record_supervision", suite, pfx)); n += 1;

        // CBC110: supervisiondate format
        e.push(regex_match(n, "supervisiondate", r"^\d{8}$", "criminal_record_supervision", suite, pfx)); n += 1;

        // ── CBC111–CBC120: Offense detail tests ───────────────────────────────

        // CBC111: offensedate not null (offense section)
        e.push(not_null(n, "offensedate", "criminal_record_offense", suite, pfx)); n += 1;

        // CBC112: offensedate format
        e.push(regex_match(n, "offensedate", r"^\d{8}$", "criminal_record_offense", suite, pfx)); n += 1;

        // CBC113: offensecode not null
        e.push(not_null(n, "offensecode", "criminal_record_offense", suite, pfx)); n += 1;

        // CBC114: offensecode max 4000 chars
        e.push(length_between(n, "offensecode", None, 4000, "criminal_record_offense", suite, pfx)); n += 1;

        // CBC115: offensedesc1 not null
        e.push(not_null(n, "offensedesc1", "criminal_record_offense", suite, pfx)); n += 1;

        // CBC116: offensedesc1 max 8000 chars
        e.push(length_between(n, "offensedesc1", None, 8000, "criminal_record_offense", suite, pfx)); n += 1;

        // CBC117: arrestdate not null (arrest section)
        e.push(not_null(n, "arrestdate", "criminal_record_arrest", suite, pfx)); n += 1;

        // CBC118: arrestdate format
        e.push(regex_match(n, "arrestdate", r"^\d{8}$", "criminal_record_arrest", suite, pfx)); n += 1;

        // CBC119: arrestagency not null
        e.push(not_null(n, "arrestagency", "criminal_record_arrest", suite, pfx)); n += 1;

        // CBC120: court not null (offense section)
        e.push(not_null(n, "court", "criminal_record_court", suite, pfx)); n += 1;

        // ── CBC121–CBC150: Additional coverage ───────────────────────────────

        // CBC121: convictionplace not null
        e.push(not_null(n, "convictionplace", "criminal_record_conviction", suite, pfx)); n += 1;

        // CBC122: convictionplace max 100 chars
        e.push(length_between(n, "convictionplace", None, 100, "criminal_record_conviction", suite, pfx)); n += 1;

        // CBC123: offensedesc2 not null
        e.push(not_null(n, "offensedesc2", "criminal_record_offense", suite, pfx)); n += 1;

        // CBC124: offensedesc2 max 4000 chars
        e.push(length_between(n, "offensedesc2", None, 4000, "criminal_record_offense", suite, pfx)); n += 1;

        // CBC125: arrestlocation not null
        e.push(not_null(n, "arrestlocation", "criminal_record_arrest", suite, pfx)); n += 1;

        // CBC126: arrestlocation max 100 chars
        e.push(length_between(n, "arrestlocation", None, 100, "criminal_record_arrest", suite, pfx)); n += 1;

        // CBC127: arrestagency max 3000 chars
        e.push(length_between(n, "arrestagency", None, 3000, "criminal_record_arrest", suite, pfx)); n += 1;

        // CBC128: prisonerid not null
        e.push(not_null(n, "prisonerid", "criminal_record_identity", suite, pfx)); n += 1;

        // CBC129: prisonerid max 100 chars
        e.push(length_between(n, "prisonerid", None, 100, "criminal_record_identity", suite, pfx)); n += 1;

        // CBC130: offendermiddlename max 100 chars
        e.push(length_between(n, "offendermiddlename", None, 100, "criminal_record_sex_offender", suite, pfx)); n += 1;

        // CBC131: offenderrace not null
        e.push(not_null(n, "offenderrace", "criminal_record_sex_offender", suite, pfx)); n += 1;

        // CBC132: offenderage not null
        e.push(not_null(n, "offenderage", "criminal_record_sex_offender", suite, pfx)); n += 1;

        // CBC133: offenderage numeric pattern
        e.push(regex_match(n, "offenderage", r"^\d{1,3}$", "criminal_record_sex_offender", suite, pfx)); n += 1;

        // CBC134: victimrelationship not null
        e.push(not_null(n, "victimrelationship", "criminal_record_sex_offender", suite, pfx)); n += 1;

        // CBC135: supervisioncounty not null
        e.push(not_null(n, "supervisioncounty", "criminal_record_supervision", suite, pfx)); n += 1;

        // CBC136: supervisioncounty max 100 chars
        e.push(length_between(n, "supervisioncounty", None, 100, "criminal_record_supervision", suite, pfx)); n += 1;

        // CBC137: supervisionstate not null
        e.push(not_null(n, "supervisionstate", "criminal_record_supervision", suite, pfx)); n += 1;

        // CBC138: supervisionstate max 100 chars
        e.push(length_between(n, "supervisionstate", None, 100, "criminal_record_supervision", suite, pfx)); n += 1;

        // CBC139: commitmentcounty not null
        e.push(not_null(n, "commitmentcounty", "criminal_record_commitment", suite, pfx)); n += 1;

        // CBC140: commitmentcounty max 100 chars
        e.push(length_between(n, "commitmentcounty", None, 100, "criminal_record_commitment", suite, pfx)); n += 1;

        // CBC141: commitmentlocation max 200 chars
        e.push(length_between(n, "commitmentlocation", None, 200, "criminal_record_commitment", suite, pfx)); n += 1;

        // CBC142: amendedcharge not null
        e.push(not_null(n, "amendedcharge", "criminal_record_amended", suite, pfx)); n += 1;

        // CBC143: amendedcharge max 200 chars
        e.push(length_between(n, "amendedcharge", None, 200, "criminal_record_amended", suite, pfx)); n += 1;

        // CBC144: amendedcasetype not null
        e.push(not_null(n, "amendedcasetype", "criminal_record_amended", suite, pfx)); n += 1;

        // CBC145: amendedcasetype max 100 chars
        e.push(length_between(n, "amendedcasetype", None, 100, "criminal_record_amended", suite, pfx)); n += 1;

        // CBC146: amendeddisposition not null
        e.push(not_null(n, "amendeddisposition", "criminal_record_amended", suite, pfx)); n += 1;

        // CBC147: amendeddisposition max 100 chars
        e.push(length_between(n, "amendeddisposition", None, 100, "criminal_record_amended", suite, pfx)); n += 1;

        // CBC148: amendeddispositiondate not null
        e.push(not_null(n, "amendeddispositiondate", "criminal_record_amended", suite, pfx)); n += 1;

        // CBC149: sentenceyyymmddd not null
        e.push(not_null(n, "sentenceyyymmddd", "criminal_record_conviction", suite, pfx)); n += 1;

        // CBC150: probationyyymmddd not null
        e.push(not_null(n, "probationyyymmddd", "criminal_record_conviction", suite, pfx)); n += 1;

        e
    }
}