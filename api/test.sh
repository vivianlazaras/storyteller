# Endpoint URL
URL="http://localhost:8442/timelines"

# Optional description (use null or a string)
DESCRIPTION="\"This is an optional description\""
# Set to `null` if you want to omit it: DESCRIPTION=null

# JSON payload
JSON=$(cat <<EOF
{
  "name": "New Timeline",
  "description": $DESCRIPTION,
  "generator": {
    "story":"5e4b87f4-bd35-4360-8edf-ab878fb36edd"
  }
}
EOF
)

# Submit via curl
curl -X POST "$URL" \
  -H "Content-Type: application/json" \
  -d "$JSON"
