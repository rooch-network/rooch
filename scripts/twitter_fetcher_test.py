#!/usr/bin/env python3

import argparse
import subprocess
import json
import time

ADDR = "0x701c21bf1c8cd5af8c42983890d8ca55e7a820171b8e744c13f2d9998bf76cc3"
COMMANDS = {
    'check_balance': f"rooch move view --function {ADDR}::tweet_fetcher::check_oracle_escrow_balance --args address:default",
    'deposit': f"rooch move run --function {ADDR}::tweet_fetcher::deposit_to_oracle_escrow --args u256:2000000000",
    'get_notification_gas': f"rooch move view --function {ADDR}::tweet_fetcher::get_notification_gas_allocation --args address:default",
    'update_notification_gas': f"rooch move run --function {ADDR}::tweet_fetcher::update_notification_gas_allocation --args u256:1000000000 --json",
    'fetch_tweet': f"rooch move run --function {ADDR}::tweet_fetcher::fetch_tweet_entry --args string:{{}}",
    'process_queue': f"rooch move run --function {ADDR}::tweet_fetcher::process_buffer_queue --json",
    'check_request': f"rooch move run --function {ADDR}::tweet_fetcher::check_request_queue",
    'check_tweet': f"rooch move view --function {ADDR}::tweet_v2::exists_tweet_object --args string:{{}}"
}

def run_command(cmd):
    result = subprocess.run(cmd.split(), capture_output=True, text=True)
    if result.returncode != 0:
        raise Exception(f"Command failed: {result.stderr}")
    return result.stdout

def check_notification_gas_balance():
    output = run_command(COMMANDS['get_notification_gas'])
    data = json.loads(output)
    balance = int(data['return_values'][0]['decoded_value'])
    if balance < 1000000000:  # 10 RGas
        print("Updating notification gas allocation...")
        run_command(COMMANDS['update_notification_gas'])

def check_and_deposit():
    output = run_command(COMMANDS['check_balance'])
    response = json.loads(output)
    
    if not response['return_values'][0]['decoded_value']:
        print("Insufficient balance, depositing funds...")
        run_command(COMMANDS['deposit'])
        print("Funds deposited")
    
    check_notification_gas_balance()

def get_request_id(process_output):
    try:
        data = json.loads(process_output)
        for event in data['output']['events']:
            if event['event_type'].endswith('::oracles::RequestAdded'):
                return event['decoded_event_data']['value']['request_id']
    except Exception as e:
        print(f"Error parsing request_id: {e}")
    return None

def check_request_status(request_id):
    cmd = f"rooch object -i {request_id}"
    output = run_command(cmd)
    data = json.loads(output)
    if 'data' in data and len(data['data']) > 0:
        request = data['data'][0]
        if 'decoded_value' in request:
            return request['decoded_value']['value'].get('response_status', 0)
    return 0

def check_request_status_with_timeout(request_id, timeout=60, interval=5):
    start_time = time.time()
    while time.time() - start_time < timeout:
        status = check_request_status(request_id)
        if status != 0:
            return status
        time.sleep(interval)
    raise TimeoutError(f"Request {request_id} did not complete within {timeout} seconds")

def fetch_tweet(tweet_id):
    check_and_deposit()
    
    print(f"Fetching tweet {tweet_id}...")
    run_command(COMMANDS['fetch_tweet'].format(tweet_id))
    
    time.sleep(2)

    print("Processing queue...")
    process_output = run_command(COMMANDS['process_queue'])
    request_id = get_request_id(process_output)
    
    time.sleep(3)
    
    if not request_id:
        raise Exception("Failed to get request_id")
    
    print(f"Request ID: {request_id}")
    check_request_status_with_timeout(request_id)
    
    print("Checking request queue...")
    run_command(COMMANDS['check_request'])
    
    timeout = 120
    start_time = time.time()
    
    while time.time() - start_time < timeout:
        output = run_command(COMMANDS['check_tweet'].format(tweet_id))
        if '"decoded_value": true' in output:
            print("Tweet fetched successfully!")
            return True
        time.sleep(5)
    
    raise TimeoutError(f"Tweet {tweet_id} not found after {timeout} seconds")

def main():
    parser = argparse.ArgumentParser(description="Tweet fetcher test script")
    parser.add_argument("tweet_id", help="Twitter ID to fetch")
    args = parser.parse_args()
    
    try:
        fetch_tweet(args.tweet_id)
    except Exception as e:
        print(f"Error: {e}")
        exit(1)

if __name__ == "__main__":
    main()