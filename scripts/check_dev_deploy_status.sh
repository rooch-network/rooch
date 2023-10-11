#!/bin/bash
# Copyright (c) RoochNetwork
# SPDX-License-Identifier: Apache-2.0

# 关键字
KEYWORD="rooch"

# 获取容器ID
CONTAINER_ID=$(docker ps -a | grep $KEYWORD | awk '{print $1}')

if [ -z "$CONTAINER_ID" ]; then
    echo "没有找到与关键字 $KEYWORD 相关的容器"
    exit 1
fi

# 获取容器状态
STATUS=$(docker inspect --format '{{.State.Status}}' $CONTAINER_ID)

if [ "$STATUS" != "running" ]; then
    echo "容器 $CONTAINER_ID 不在运行状态，尝试清理数据，重新启动，并部署examples..."
    echo "开始清理数据"
    rooch server clean -n dev
    docker start $CONTAINER_ID
    if [ $? -eq 0 ]; then
        echo "容器 $CONTAINER_ID 已成功重新启动"
        echo "重新部署examples"
        for dir in ../examples/*/; do
            dir=${dir%*/}
            name_addr=$(basename $dir)
            echo $name_addr
            rooch move build -p "$dir" --named-addresses rooch_examples=default,$name_addr=default
            rooch move publish --named-addresses rooch_examples=default,$name_addr=default -p ../examples/$name_addr/
        done
    else
        echo "容器 $CONTAINER_ID 启动失败，请查看原因"
    fi
else
    echo "容器 $CONTAINER_ID 正在运行"
fi
