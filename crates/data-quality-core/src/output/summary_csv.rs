use crate::error::DqError;
use crate::expectations::GeneratedFrom;
use crate::output::{DqOutputFile, DqOutputFormat};

/// Build a summary CSV file from a DqSuiteSet.
pub fn build_summary_csv(suite_set: &crate::DqSuiteSet) -> Result<DqOutputFile, DqError> {
    let contract_name = suite_set.contract_name.as_deref().unwrap_or("unnamed");
    let filename = format!("{}/summary.csv", contract_name);

    let mut wtr = csv::Writer::from_writer(Vec::new());

    // Write header
    wtr.write_record(&[
        "test_id",
        "suite_name",
        "expectation_type",
        "category",
        "contract_field",
        "contract_name",
        "generated_from",
        "kwargs_json",
    ])
    .map_err(|e| DqError::SerializationError(e.to_string()))?;

    // Write all baseline suite expectations
    for suite in &suite_set.baseline_suites {
        for exp in &suite.expectations {
            let generated_from = match &exp.meta.generated_from {
                GeneratedFrom::Baseline => "baseline".to_string(),
                GeneratedFrom::ContractSpecific { reason } => {
                    format!("contract_specific:{}", reason)
                }
            };
            let kwargs_json = serde_json::to_string(&exp.kwargs)
                .unwrap_or_else(|_| "{}".to_string());
            wtr.write_record(&[
                &exp.meta.test_id,
                &suite.name,
                &exp.expectation_type,
                &exp.meta.category,
                exp.meta.contract_field.as_deref().unwrap_or(""),
                exp.meta.contract_name.as_deref().unwrap_or(""),
                &generated_from,
                &kwargs_json,
            ])
            .map_err(|e| DqError::SerializationError(e.to_string()))?;
        }
    }

    // Write all contract-specific suite expectations
    for suite in &suite_set.contract_suites {
        for exp in &suite.expectations {
            let generated_from = match &exp.meta.generated_from {
                GeneratedFrom::Baseline => "baseline".to_string(),
                GeneratedFrom::ContractSpecific { reason } => {
                    format!("contract_specific:{}", reason)
                }
            };
            let kwargs_json = serde_json::to_string(&exp.kwargs)
                .unwrap_or_else(|_| "{}".to_string());
            wtr.write_record(&[
                &exp.meta.test_id,
                &suite.name,
                &exp.expectation_type,
                &exp.meta.category,
                exp.meta.contract_field.as_deref().unwrap_or(""),
                exp.meta.contract_name.as_deref().unwrap_or(""),
                &generated_from,
                &kwargs_json,
            ])
            .map_err(|e| DqError::SerializationError(e.to_string()))?;
        }
    }

    let content = wtr
        .into_inner()
        .map_err(|e| DqError::SerializationError(e.to_string()))?;

    Ok(DqOutputFile {
        filename,
        suite_name: "summary".to_string(),
        content,
        format: DqOutputFormat::SummaryCsv,
    })
}
