use mos_integration_test_runner::run_test;

datatest_stable::harness!(run_test, "tests", r".*\.(mvir|move)$");
