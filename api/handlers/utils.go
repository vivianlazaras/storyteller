package handlers

import (
	"regexp"
	"net/http"
	"github.com/gin-gonic/gin"
	"github.com/google/uuid"

)

type Result struct {
	Status		int
	Message 	string
}

func (result *Result) GinResult(c *gin.Context) {
	c.JSON(result.Status, gin.H{"error": result.Message})
}

func NewResult(status int, message string) Result {
	return Result {
		Status: status,
		Message: message,
	}
}

func (result *Result) IsError() bool {
	return result.Status != http.StatusOK
}

// RemoveJavaScriptTags removes all <script>...</script> blocks from a Markdown string.
/// This is nessisary to prevent javascript injections that could potentially be used to
/// gain access to other peoples systems.
func RemoveJavaScriptTags(markdown string) string {
	// (?i) makes it case-insensitive, (?s) allows . to match newlines
	re := regexp.MustCompile(`(?is)<script.*?>.*?</script>`)
	return re.ReplaceAllString(markdown, "")
}

func GetIDParam(idtext string) (*uuid.UUID, Result) {

	if idtext == "" {
		return nil, NewResult(http.StatusBadRequest, "missing task id")
	}
	parsedID, parseErr := uuid.Parse(idtext);
	if parseErr != nil {
		return nil, NewResult(http.StatusBadRequest, "improperly formatted UUID")
	}

	return &parsedID, NewResult(http.StatusOK, "successfully parsed ID")
}