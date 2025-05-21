package handlers

import (
	"github.com/google/uuid"
)

// this function looks for a group named null with no members
// if not found it will create it, this will be used as the default value
// when creating metadata for unshared objects. This is a workaround because go tries to
// insert '' for empty string
func getNullGroup() {
	var results []uuid.UUID

	err := db.Raw(`
		SELECT usergroups.id, COUNT(grouprel.userid) AS user_count
		FROM usergroups
		LEFT JOIN grouprel ON usergroups.id = grouprel.groupid
		WHERE usergroups.name = ?
		GROUP BY usergroups.id
		HAVING COUNT(grouprel.userid) = 0
	`, "null").Scan(&results).Error

}