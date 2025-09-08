#!/bin/bash -e

echo "🔧 Removing deprecated --ca_contents argument from Edgegap app version"
echo "✨ This prepares your app for LetsEncrypt certificate usage"
echo

appname=$1
appver=$2

if [ -z "$appname" ] || [ -z "$appver" ] ; then
	echo "Usage: $0 <appname> <appversion>"
	echo
	echo "This script removes the deprecated --ca_contents argument from your Edgegap app version."
	echo "After running this script, your app will use LetsEncrypt certificates with the system trust store."
	exit 1
fi

if [ -z "$EDGEGAP_API_KEY" ] ; then
	echo "Ensure your EDGEGAP_API_KEY is set"
	exit 2
fi

# Set arguments to empty string to remove all arguments including --ca_contents
body="{\"arguments\": \"\"}"

url="https://api.edgegap.com/v1/app/$appname/version/$appver"
echo "🔧 Sending PATCH command to $url to remove --ca_contents argument"
echo 
curl -X PATCH "$url" -H "Content-Type: application/json" -H "Authorization: $EDGEGAP_API_KEY" -d "$body" -o -
echo
if [ $? == 0 ]; then
	echo "✅ SUCCESS: Removed --ca_contents argument from $appname version $appver" 
	echo "🎉 Your app is now ready to use LetsEncrypt certificates with the system trust store!"
	echo "ℹ️  No custom CA configuration is needed anymore."
else
	echo "❌ Failed to remove --ca_contents argument."
	echo "ℹ️  Check your EDGEGAP_API_KEY and ensure the app name and version are correct."
fi