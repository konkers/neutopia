#!/bin/bash
if (echo ${GITHUB_REF} | egrep -q '^refs/tags/v[0-9]+.[0-9]+.[0-9]+-.*'); then
    DEPLOY_SITE="beta-neutopia-run"
elif (echo ${GITHUB_REF} | egrep -q '^refs/tags/v[0-9]+.[0-9]+.[0-9]+'); then
    DEPLOY_SITE="neutopia-run"
else
    DEPLOY_SITE="dev-neutopia-run"
fi

case $1 in
"DEPLOY_SITE")
    echo "::set-env name=DEPLOY_SITE::${DEPLOY_SITE}"
    ;;
esac