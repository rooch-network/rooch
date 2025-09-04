#!/bin/bash

# Rooch Testnet Rollback Script
# This script helps you perform a rollback operation on the rooch testnet node

set -e

NAMESPACE="testnet"
STATEFULSET_NAME="rooch-testnet"
TOOLBOX_POD_NAME="rooch-data-toolbox"
TOOLBOX_POD_FILE="rooch-node/rooch-data-toolbox-pod.yaml"

echo "=== Rooch Testnet Rollback Script ==="
echo ""

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check if kubectl is available
if ! command_exists kubectl; then
    echo "Error: kubectl is not installed or not in PATH"
    exit 1
fi

# Function to wait for pod to be ready
wait_for_pod() {
    local pod_name=$1
    local namespace=$2
    echo "Waiting for pod $pod_name to be ready..."
    kubectl wait --for=condition=Ready pod/$pod_name -n $namespace --timeout=300s
}

# Function to check if pod exists
pod_exists() {
    local pod_name=$1
    local namespace=$2
    kubectl get pod $pod_name -n $namespace >/dev/null 2>&1
}

# Function to check if statefulset exists
statefulset_exists() {
    local ss_name=$1
    local namespace=$2
    kubectl get statefulset $ss_name -n $namespace >/dev/null 2>&1
}

echo "Step 1: Checking current state..."
if statefulset_exists $STATEFULSET_NAME $NAMESPACE; then
    echo "✓ StatefulSet $STATEFULSET_NAME found in namespace $NAMESPACE"
    REPLICAS=$(kubectl get statefulset $STATEFULSET_NAME -n $NAMESPACE -o jsonpath='{.spec.replicas}')
    echo "  Current replicas: $REPLICAS"
else
    echo "✗ StatefulSet $STATEFULSET_NAME not found in namespace $NAMESPACE"
    exit 1
fi

echo ""
echo "Step 2: Scaling down StatefulSet to 0 replicas..."
kubectl scale statefulset $STATEFULSET_NAME --replicas=0 -n $NAMESPACE
echo "✓ StatefulSet scaled down"

echo ""
echo "Step 3: Waiting for pods to terminate..."
kubectl wait --for=delete pod -l app=$STATEFULSET_NAME -n $NAMESPACE --timeout=300s
echo "✓ All pods terminated"

echo ""
echo "Step 4: Creating data toolbox pod..."
if pod_exists $TOOLBOX_POD_NAME $NAMESPACE; then
    echo "Data toolbox pod already exists, deleting it first..."
    kubectl delete pod $TOOLBOX_POD_NAME -n $NAMESPACE
    sleep 5
fi

kubectl apply -f $TOOLBOX_POD_FILE
echo "✓ Data toolbox pod created"

echo ""
echo "Step 5: Waiting for data toolbox pod to be ready..."
wait_for_pod $TOOLBOX_POD_NAME $NAMESPACE
echo "✓ Data toolbox pod is ready"

echo ""
echo "=== Data Toolbox Pod is Ready ==="
echo ""
echo "You can now exec into the data toolbox pod to perform maintenance operations:"
echo ""
echo "kubectl exec -it $TOOLBOX_POD_NAME -n $NAMESPACE -- /bin/sh"
echo ""
echo "Available maintenance commands:"
echo "  - Rollback: /rooch/rooch db rollback --tx-order <YOUR_TX_ORDER> -d /root/.rooch -n test"
echo "  - Revert:   /rooch/rooch db revert-tx --tx-order <YOUR_TX_ORDER> -d /root/.rooch -n test"
echo "  - Repair:   /rooch/rooch db repair -d /root/.rooch -n test"
echo "  - Check tx: /rooch/rooch db get-tx-by-order --tx-order <ORDER> -d /root/.rooch -n test"
echo "  - Anomalies:/rooch/rooch db list-anomaly -d /root/.rooch -n test"
echo ""
echo "After maintenance is complete, run the following to clean up and restart:"
echo "./data-maintenance-script.sh cleanup"
echo ""

# Check if cleanup argument is provided
if [ "$1" = "cleanup" ]; then
    echo "=== Cleanup Phase ==="
    echo ""
    echo "Step 1: Deleting data toolbox pod..."
    kubectl delete pod $TOOLBOX_POD_NAME -n $NAMESPACE
    echo "✓ Data toolbox pod deleted"
    
    echo ""
    echo "Step 2: Scaling StatefulSet back to 1 replica..."
    kubectl scale statefulset $STATEFULSET_NAME --replicas=1 -n $NAMESPACE
    echo "✓ StatefulSet scaled up"
    
    echo ""
    echo "Step 3: Waiting for main pod to be ready..."
    wait_for_pod $STATEFULSET_NAME-0 $NAMESPACE
    echo "✓ Main pod is ready"
    
    echo ""
    echo "=== Rollback Complete ==="
    echo "The rooch testnet node has been restarted and should be running normally."
    echo "You can check the logs with:"
    echo "kubectl logs -f $STATEFULSET_NAME-0 -n $NAMESPACE"
fi
