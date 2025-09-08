#!/bin/bash -e

echo "⚠️  DEPRECATED: This script is deprecated since LetsEncrypt certificates are now used."
echo "⚠️  With LetsEncrypt certificates, no custom CA setup is needed."
echo "⚠️  The system trust store automatically handles certificate verification."
echo "⚠️  This script is kept for legacy compatibility only."
echo

appname=$1
appver=$2
cafile=$3

if [ ! -f "$cafile" ] || [ -z "$appname" ] || [ -z "$appver" ] ; then
	echo "Usage: $0 <appname> <appversion> <path to rootCA.pem file>"
	echo
	echo "DEPRECATED: Consider migrating to LetsEncrypt certificates instead."
	exit 1
fi

if [ -z "$EDGEGAP_API_KEY" ] ; then
	echo "Ensure your EDGEGAP_API_KEY is set"
	exit 2
fi

body="{\"arguments\": \"--ca_contents '$(cat "$cafile" | tr -d '\n')'\"}"

#echo "Setting body to: $body"
url="https://api.edgegap.com/v1/app/$appname/version/$appver"
#echo "url=$url"
echo "🔧 Sending PATCH command to $url"
echo 
curl -X PATCH "$url" -H "Content-Type: application/json" -H "Authorization: $EDGEGAP_API_KEY" -d "$body" -o -
echo
if [ $? == 0 ]; then
	echo "✅ OK. Deployments of $appname at version $appver will have --ca_contents '<...contents...>' passed as arguments." 
	echo "⚠️  REMINDER: Consider migrating to LetsEncrypt certificates to eliminate the need for custom CA setup."
else
	echo "❌ Oh no."
fi
