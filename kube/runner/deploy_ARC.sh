kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.12.0/cert-manager.yaml

kubectl apply -f https://github.com/actions/actions-runner-controller/releases/download/v0.27.3/actions-runner-controller.yaml

kubectl create secret generic controller-manager -n actions-runner-system --from-literal=github_token=xxx

