# Endpoint URL
URL="http://localhost:8000/stories"

# Optional description (use null or a string)
DESCRIPTION="\"This is an optional description\""
# Set to `null` if you want to omit it: DESCRIPTION=null

# JSON payload
JSON=$(cat <<EOF
{
  "name": "My Fragment",
  "description": $DESCRIPTION,
  "render": "markdown",
  "content": "# Hello World\nThis is a fragment."
}
EOF
)

# Submit via curl
curl -X POST "$URL" \
  -H "Content-Type: application/json" \
  -d "$JSON"
