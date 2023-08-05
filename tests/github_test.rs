use chrono::{NaiveDate, TimeZone, Utc};
use duckdb::arrow::{
    array::{as_primitive_array, as_string_array, Date32Array, Float64Array, StringArray},
    datatypes::Date32Type,
};
use std::{fs, path::Path};
use wallowa::{
    db::open_db_pool,
    github::fetch::{fetch_commits, fetch_pulls, ResponseInfo},
    github::queries::merged_pr_duration_rolling_daily_average,
};

#[test]
fn test_github_commits() {
    // Load the test data
    let cargo_root = std::env::var("CARGO_MANIFEST_DIR").expect("Missing CARGO_MANIFEST_DIR");
    let path = Path::new(cargo_root.as_str()).join("tests/github.commits.json");
    let file_contents = fs::read_to_string(path).expect("Unable to read file");

    // Prep the test DB
    let pool = open_db_pool(":memory:", 1).expect("Unable to open db");

    let response_info = ResponseInfo {
        request_url: "https://api.github.com/repos/octocat/Hello-World/commits".to_string(),
        owner: "octocat".to_string(),
        repo: "Hello-World".to_string(),
        status: 200,
        response: file_contents.clone(),
        watermark: Utc::now(),
    };
    fetch_commits(pool.clone(), &vec![response_info]).expect("Unable to fetch commits");
    let sql = "SELECT count(*) FROM wallowa_raw_data;";
    let count: usize = pool
        .get()
        .unwrap()
        .query_row_and_then(sql, [], |row| row.get(0))
        .expect("Unable to query count from wallowa_raw_data");
    assert_eq!(count, 1);
}

#[test]
fn test_github_pulls() {
    // Load the test data
    let cargo_root = std::env::var("CARGO_MANIFEST_DIR").expect("Missing CARGO_MANIFEST_DIR");
    let path = Path::new(cargo_root.as_str()).join("tests/github.pulls.json");
    let file_contents = fs::read_to_string(path).expect("Unable to read file");

    // Prep the test DB
    let pool = open_db_pool(":memory:", 1).expect("Unable to open db");

    let response_info = ResponseInfo {
        request_url: "https://api.github.com/repos/octocat/Hello-World/pulls".to_string(),
        owner: "octocat".to_string(),
        repo: "Hello-World".to_string(),
        status: 200,
        response: file_contents.clone(),
        watermark: Utc::now(),
    };
    fetch_pulls(&pool, &vec![response_info]).expect("Unable to fetch pulls");
    let sql = "SELECT count(*) FROM wallowa_raw_data;";
    let count: usize = pool
        .get()
        .unwrap()
        .query_row_and_then(sql, [], |row| row.get(0))
        .expect("Unable to query count from wallowa_raw_data");
    assert_eq!(count, 1);
}

/// Use a known set of fake pull request data to test that
/// the average pull request query works as expected.
/// TODO fix to query directly against raw data since the `github_commit` table no longer exists
#[test]
fn test_avg_pr_query() {
    // Setup expected data to compare to
    let mut current_day = NaiveDate::from_ymd_opt(2011, 1, 1).unwrap();
    let mut day_vec = vec![];
    for _i in 0..31 {
        day_vec.push(Date32Type::from_naive_date(current_day));
        current_day = current_day.succ_opt().unwrap();
    }
    let expected_days = Date32Array::from(day_vec);
    let expected_durations = Float64Array::from(vec![
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(1.2855555555555556),
        Some(1.2855555555555556),
        Some(1.2855555555555556),
        Some(1.2855555555555556),
        Some(1.2855555555555556),
        Some(1.2855555555555556),
    ]);
    let expected_repos = StringArray::from(vec![
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some("octocat/Hello-World"),
        Some("octocat/Hello-World"),
        Some("octocat/Hello-World"),
        Some("octocat/Hello-World"),
        Some("octocat/Hello-World"),
        Some("octocat/Hello-World"),
    ]);

    // Load the test data
    let cargo_root = std::env::var("CARGO_MANIFEST_DIR").expect("Missing CARGO_MANIFEST_DIR");
    let path = Path::new(cargo_root.as_str()).join("tests/github.pulls.json");
    let file_contents = fs::read_to_string(path).expect("Unable to read file");

    // Prep the test DB
    let pool = open_db_pool(":memory:", 1).expect("Unable to open db");
    let response_info = ResponseInfo {
        request_url: "https://api.github.com/repos/octocat/Hello-World/pulls".to_string(),
        owner: "octocat".to_string(),
        repo: "Hello-World".to_string(),
        status: 200,
        response: file_contents.clone(),
        watermark: Utc::now(),
    };
    fetch_pulls(&pool, &vec![response_info]).expect("Unable to fetch pulls");

    // Run the query being tested
    let results = merged_pr_duration_rolling_daily_average(
        &pool,
        Utc.with_ymd_and_hms(2011, 1, 1, 0, 0, 0)
            .unwrap()
            .fixed_offset(),
        Utc.with_ymd_and_hms(2011, 1, 31, 0, 0, 0)
            .unwrap()
            .fixed_offset(),
        &vec![],
    )
    .expect("Unable to calculate average duration for merged PRs");

    // Check the results
    let batch = &results[0];
    let days: &Date32Array = as_primitive_array(
        batch
            .column_by_name("day")
            .expect("`day` column not returned"),
    );
    assert_eq!(expected_days, *days, "incorrect `day` column returned");
    let durations: &Float64Array = as_primitive_array(
        batch
            .column_by_name("duration")
            .expect("`duration` column not returned"),
    );
    assert_eq!(
        expected_durations, *durations,
        "incorrect `duration` column returned"
    );
    let repos = as_string_array(
        batch
            .column_by_name("repo")
            .expect("`repo` column not returned"),
    );
    assert_eq!(expected_repos, *repos, "incorrect `repo` column returned");
}
