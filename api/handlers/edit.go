package handlers

import (
	"strings"
	"github.com/gin-gonic/gin"
)
func RegisterEditRoutes(r *gin.Engine) *gin.Engine {
	return r
}

type LineChange struct {
	Change		string	`json:"change"`
	Previous	string	`json:"prev"`
	Value		string	`json:"value"`

}

func CalcDiff(original, newtext *string) []LineChange {
	origLines := strings.Split(*original, "\n")
	newLines := strings.Split(*newtext, "\n")

	maxLen := len(origLines)
	if len(newLines) > maxLen {
		maxLen = len(newLines)
	}

	var changes []LineChange

	for i := 0; i < maxLen; i++ {
		var origLine, newLine string
		if i < len(origLines) {
			origLine = origLines[i]
		}
		if i < len(newLines) {
			newLine = newLines[i]
		}

		switch {
		case i >= len(origLines):
			changes = append(changes, LineChange{
				Change:   "added",
				Previous: "",
				Value:    newLine,
			})
		case i >= len(newLines):
			changes = append(changes, LineChange{
				Change:   "removed",
				Previous: origLine,
				Value:    "",
			})
		case origLine == newLine:
			changes = append(changes, LineChange{
				Change:   "unchanged",
				Previous: origLine,
				Value:    newLine,
			})
		default:
			changes = append(changes, LineChange{
				Change:   "changed",
				Previous: origLine,
				Value:    newLine,
			})
		}
	}

	return changes
}

