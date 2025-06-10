package handlers

import (
	"github.com/google/uuid"
	"gorm.io/gorm"
	"github.com/vivianlazaras/storyteller/db"
	"github.com/vivianlazaras/storyteller/model"
)

// this function looks for a group named null with no members
// if not found it will create it, this will be used as the default value
// when creating metadata for unshared objects. This is a workaround because go tries to
// insert '' for empty string
func getNullGroup() (string, error) {
	var results []string

	err := db.DB.Raw(`
		SELECT groups.id
		FROM groups
		LEFT JOIN grouprel ON groups.id = grouprel.groupid
		WHERE groups.name = ?
		GROUP BY groups.id
		HAVING COUNT(grouprel.userid) = 0
	`, "null").Scan(&results).Error

	if err != nil {
		return "", err
	}
	if len(results) > 0 {
		return results[0], nil
	}

	var nullGroup = model.Group {
		ID: uuid.New().String(),
		Name: "null",
		Description: "The default group assigned when creating metadata object, this group should never have any members",
	}

	dberr := db.DB.Create(nullGroup).Error
	return nullGroup.ID, dberr
}

func GetGroupsForUser(db *gorm.DB, userID uuid.UUID) ([]model.Group, error) {
	var groups []model.Group

	err := db.Table("groups").
		Joins("JOIN grouprel ON groups.id = grouprel.groupid").
		Where("grouprel.userid = ?", userID).
		Scan(&groups).Error

	if err != nil {
		return nil, err
	}
	return groups, nil
}