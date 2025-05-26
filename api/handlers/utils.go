package handlers

import (
	"fmt"
	"regexp"
)

// RemoveJavaScriptTags removes all <script>...</script> blocks from a Markdown string.
/// This is nessisary to prevent javascript injections that could potentially be used to
/// gain access to other peoples systems.
func RemoveJavaScriptTags(markdown string) string {
	// (?i) makes it case-insensitive, (?s) allows . to match newlines
	re := regexp.MustCompile(`(?is)<script.*?>.*?</script>`)
	return re.ReplaceAllString(markdown, "")
}