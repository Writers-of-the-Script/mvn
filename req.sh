#!/bin/bash

source .env
curl -H "Authorization: Bearer $MASTER_KEY" -H "Content-Type: application/json" "$@"
