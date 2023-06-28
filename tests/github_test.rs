use chrono::{TimeZone, Utc};
use float_cmp::approx_eq;
use opsql::{
    db::open_db_pool,
    queries::github::{
        count_commits, count_pulls, merged_pr_duration_30_day_rolling_avg_hours, DurationByDay,
    },
    sources::github::{fetch_commits, fetch_pulls, load_commits, load_pulls, ResponseInfo},
};
use std::{fs, path::Path};

#[test]
fn test_github_commits() {
    // Load the test data
    let cargo_root = std::env::var("CARGO_MANIFEST_DIR").expect("Missing CARGO_MANIFEST_DIR");
    let path = Path::new(cargo_root.as_str()).join("tests/github.commits.json");
    let file_contents = fs::read_to_string(path).expect("Unable to read file");

    // Prep the test DB
    let pool = open_db_pool(":memory:", 1).expect("Unable to open db");

    let response_info = ResponseInfo {
        request_url: "https://api.github.com/repos/octocat/Spoon-Knife/commits".to_string(),
        owner: "octocat".to_string(),
        repo: "Spoon-Knife".to_string(),
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

    load_commits(pool.clone()).expect("Unable to load commits");

    let sql = "SELECT count(*) FROM github_commit;";
    let count: usize = pool
        .get()
        .unwrap()
        .query_row_and_then(sql, [], |row| row.get(0))
        .expect("Unable to query count from commits");
    assert_eq!(count, 3);

    let results = count_commits(pool).expect("Unable to count commits");
    let row = results.get(0).expect("Unable to get first row");
    assert_eq!(row.owner, "octocat".to_string());
    assert_eq!(row.repo, "Spoon-Knife".to_string());
    assert_eq!(row.count, 3);
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
        request_url: "https://api.github.com/repos/octocat/Spoon-Knife/pulls".to_string(),
        owner: "octocat".to_string(),
        repo: "HelloWorld".to_string(),
        status: 200,
        response: file_contents.clone(),
        watermark: Utc::now(),
    };
    fetch_pulls(pool.clone(), &vec![response_info]).expect("Unable to fetch pulls");
    let sql = "SELECT count(*) FROM wallowa_raw_data;";
    let count: usize = pool
        .get()
        .unwrap()
        .query_row_and_then(sql, [], |row| row.get(0))
        .expect("Unable to query count from wallowa_raw_data");
    assert_eq!(count, 1);

    load_pulls(pool.clone()).expect("Unable to load pulls");

    let sql = "SELECT count(*) FROM github_pull;";
    let count: usize = pool
        .get()
        .unwrap()
        .query_row_and_then(sql, [], |row| row.get(0))
        .expect("Unable to query count from pulls");
    assert_eq!(count, 1);

    let results = count_pulls(pool).expect("Unable to count pulls");
    let row = results.get(0).expect("Unable to get first row");
    assert_eq!(row.owner, "octocat".to_string());
    assert_eq!(row.repo, "HelloWorld".to_string());
    assert_eq!(row.count, 1);
}

/// Use a known set of fake pull request data to test that
/// the average pull request query works as expected.
#[test]
fn test_avg_pr_query() {
    let pool = open_db_pool(":memory:", 1).expect("Unable to open db");

    // Wrap this data setup in a block so that `conn` goes out of scope and is returned to the `pool`
    {
        let conn = pool.get().unwrap();
        conn.execute(r#"
INSERT INTO github_pull 
(owner, repo, number, created_at, closed_at, merged_at, state)
VALUES
('fakeowner', 'FakeRepo', '1', '2020-01-06 12:51:00-00', NULL, NULL, 'open'), -- Duration: NULL
('fakeowner', 'FakeRepo', '2', '2020-01-06 14:00:00-00', '2020-01-07 10:55:00-00', '2020-01-07 10:50:00-00', 'closed'), -- Duration: 20.83333333
('fakeowner', 'FakeRepo', '3', '2020-01-08 13:25:00-00', NULL, '2020-01-10 15:43:00-00', 'open'), -- Duration: 50.3
('fakeowner', 'FakeRepo', '4', '2020-01-13 09:37:00-00', '2020-01-13 14:41:00-00', '2020-01-13 14:54:00-00', 'closed'), -- Duration: 5.283333333
('fakeowner', 'FakeRepo', '5', '2020-01-13 13:28:00-00', '2020-01-14 08:56:00-00', '2020-01-14 08:32:00-00', 'closed'), -- Duration: 19.06666667
('fakeowner', 'FakeRepo', '6', '2020-01-14 17:12:00-00', '2020-01-16 16:22:00-00', '2020-01-16 15:47:00-00', 'closed'), -- Duration: 46.58333333
('fakeowner', 'FakeRepo', '7', '2020-01-16 18:05:00-00', NULL, '2020-01-17 10:33:00-00', 'open'), -- Duration: 16.46666667
('fakeowner', 'FakeRepo', '8', '2020-01-21 14:31:00-00', NULL, '2020-01-21 16:53:00-00', 'open'), -- Duration: 2.366666667
('fakeowner', 'FakeRepo', '9', '2020-01-24 11:14:00-00', '2020-01-28 09:45:00-00', '2020-01-27 10:23:00-00', 'closed'), -- Duration: 71.15
('fakeowner', 'FakeRepo', '10', '2020-02-05 10:32:00-00', NULL, '2020-02-05 16:36:00-00', 'open'), -- Duration: 6.066666667
('fakeowner', 'FakeRepo', '11', '2020-02-10 16:04:00-00', NULL, NULL, 'open'), -- Duration: NULL
('fakeowner', 'FakeRepo', '12', '2020-02-10 16:23:00-00', '2020-02-11 09:27:00-00', NULL, 'closed'); -- Duration: NULL
"#,
            []).expect("Unable to prepare insert statement");
    }

    // These expected results are calculated by hand
    let expected = vec![
        DurationByDay {
            date: Utc.with_ymd_and_hms(2020, 01, 06, 0, 0, 0).unwrap(),
            duration: None,
        },
        DurationByDay {
            date: Utc.with_ymd_and_hms(2020, 01, 07, 0, 0, 0).unwrap(),
            duration: Some(20.83333333 / 24.0),
        },
        DurationByDay {
            date: Utc.with_ymd_and_hms(2020, 01, 08, 0, 0, 0).unwrap(),
            duration: Some(20.83333333 / 24.0),
        },
        DurationByDay {
            date: Utc.with_ymd_and_hms(2020, 01, 09, 0, 0, 0).unwrap(),
            duration: Some(20.83333333 / 24.0),
        },
        DurationByDay {
            date: Utc.with_ymd_and_hms(2020, 01, 10, 0, 0, 0).unwrap(),
            duration: Some(35.56666665 / 24.0),
        },
        DurationByDay {
            date: Utc.with_ymd_and_hms(2020, 01, 11, 0, 0, 0).unwrap(),
            duration: Some(35.56666665 / 24.0),
        },
        DurationByDay {
            date: Utc.with_ymd_and_hms(2020, 01, 12, 0, 0, 0).unwrap(),
            duration: Some(35.56666665 / 24.0),
        },
        DurationByDay {
            date: Utc.with_ymd_and_hms(2020, 01, 13, 0, 0, 0).unwrap(),
            duration: Some(25.47222221 / 24.0),
        },
        DurationByDay {
            date: Utc.with_ymd_and_hms(2020, 01, 14, 0, 0, 0).unwrap(),
            duration: Some(23.87083333 / 24.0),
        },
        DurationByDay {
            date: Utc.with_ymd_and_hms(2020, 01, 15, 0, 0, 0).unwrap(),
            duration: Some(23.87083333 / 24.0),
        },
        DurationByDay {
            date: Utc.with_ymd_and_hms(2020, 01, 16, 0, 0, 0).unwrap(),
            duration: Some(28.41333333 / 24.0),
        },
        DurationByDay {
            date: Utc.with_ymd_and_hms(2020, 01, 17, 0, 0, 0).unwrap(),
            duration: Some(26.42222222 / 24.0),
        },
        DurationByDay {
            date: Utc.with_ymd_and_hms(2020, 01, 18, 0, 0, 0).unwrap(),
            duration: Some(26.42222222 / 24.0),
        },
        DurationByDay {
            date: Utc.with_ymd_and_hms(2020, 01, 19, 0, 0, 0).unwrap(),
            duration: Some(26.42222222 / 24.0),
        },
        DurationByDay {
            date: Utc.with_ymd_and_hms(2020, 01, 20, 0, 0, 0).unwrap(),
            duration: Some(26.42222222 / 24.0),
        },
        DurationByDay {
            date: Utc.with_ymd_and_hms(2020, 01, 21, 0, 0, 0).unwrap(),
            duration: Some(22.98571428 / 24.0),
        },
        DurationByDay {
            date: Utc.with_ymd_and_hms(2020, 01, 22, 0, 0, 0).unwrap(),
            duration: Some(22.98571428 / 24.0),
        },
        DurationByDay {
            date: Utc.with_ymd_and_hms(2020, 01, 23, 0, 0, 0).unwrap(),
            duration: Some(22.98571428 / 24.0),
        },
        DurationByDay {
            date: Utc.with_ymd_and_hms(2020, 01, 24, 0, 0, 0).unwrap(),
            duration: Some(22.98571428 / 24.0),
        },
        DurationByDay {
            date: Utc.with_ymd_and_hms(2020, 01, 25, 0, 0, 0).unwrap(),
            duration: Some(22.98571428 / 24.0),
        },
        DurationByDay {
            date: Utc.with_ymd_and_hms(2020, 01, 26, 0, 0, 0).unwrap(),
            duration: Some(22.98571428 / 24.0),
        },
        DurationByDay {
            date: Utc.with_ymd_and_hms(2020, 01, 27, 0, 0, 0).unwrap(),
            duration: Some(29.00625 / 24.0),
        },
        DurationByDay {
            date: Utc.with_ymd_and_hms(2020, 01, 28, 0, 0, 0).unwrap(),
            duration: Some(29.00625 / 24.0),
        },
        DurationByDay {
            date: Utc.with_ymd_and_hms(2020, 01, 29, 0, 0, 0).unwrap(),
            duration: Some(29.00625 / 24.0),
        },
        DurationByDay {
            date: Utc.with_ymd_and_hms(2020, 01, 30, 0, 0, 0).unwrap(),
            duration: Some(29.00625 / 24.0),
        },
        DurationByDay {
            date: Utc.with_ymd_and_hms(2020, 01, 31, 0, 0, 0).unwrap(),
            duration: Some(29.00625 / 24.0),
        },
        DurationByDay {
            date: Utc.with_ymd_and_hms(2020, 02, 01, 0, 0, 0).unwrap(),
            duration: Some(29.00625 / 24.0),
        },
        DurationByDay {
            date: Utc.with_ymd_and_hms(2020, 02, 02, 0, 0, 0).unwrap(),
            duration: Some(29.00625 / 24.0),
        },
        DurationByDay {
            date: Utc.with_ymd_and_hms(2020, 02, 03, 0, 0, 0).unwrap(),
            duration: Some(29.00625 / 24.0),
        },
        DurationByDay {
            date: Utc.with_ymd_and_hms(2020, 02, 04, 0, 0, 0).unwrap(),
            duration: Some(29.00625 / 24.0),
        },
        DurationByDay {
            date: Utc.with_ymd_and_hms(2020, 02, 05, 0, 0, 0).unwrap(),
            duration: Some(26.4574074 / 24.0),
        },
        DurationByDay {
            date: Utc.with_ymd_and_hms(2020, 02, 06, 0, 0, 0).unwrap(),
            duration: Some(26.4574074 / 24.0),
        },
        DurationByDay {
            date: Utc.with_ymd_and_hms(2020, 02, 07, 0, 0, 0).unwrap(),
            duration: Some(26.4574074 / 24.0),
        },
        DurationByDay {
            date: Utc.with_ymd_and_hms(2020, 02, 08, 0, 0, 0).unwrap(),
            duration: Some(26.4574074 / 24.0),
        },
        DurationByDay {
            date: Utc.with_ymd_and_hms(2020, 02, 09, 0, 0, 0).unwrap(),
            duration: Some(26.4574074 / 24.0),
        },
        DurationByDay {
            date: Utc.with_ymd_and_hms(2020, 02, 10, 0, 0, 0).unwrap(),
            duration: Some(26.4574074 / 24.0),
        },
        DurationByDay {
            date: Utc.with_ymd_and_hms(2020, 02, 11, 0, 0, 0).unwrap(),
            duration: Some(26.4574074 / 24.0),
        },
    ];

    // Test the first 31 days of the table
    let results = merged_pr_duration_30_day_rolling_avg_hours(
        pool,
        "fakeowner",
        "FakeRepo",
        Utc.with_ymd_and_hms(2020, 02, 05, 0, 0, 0).unwrap(),
    )
    .expect("Unable to calculate average duration for merged PRs");
    let expected_first_31_days = &expected[0..31];
    assert_eq!(expected_first_31_days.len(), results.len());
    for (expected, actual) in expected.iter().zip(results.iter()) {
        assert_eq!(expected.date, actual.date);
        if expected.duration.is_none() && actual.duration.is_none() {
            // Passed, they match
            continue;
        } else if expected.duration.is_none() && actual.duration.is_some() {
            // Failed, they don't match
            assert!(false);
        } else if expected.duration.is_some() && actual.duration.is_none() {
            // Failed, they don't match
            assert!(false);
        } else {
            // Both are Some, loosely compare the values
            assert!(approx_eq!(
                f64,
                expected.duration.unwrap(),
                actual.duration.unwrap(),
                epsilon = 0.000000001
            ));
        }
    }

    // Test the last 31 days of the table
}
