// Code generated by gorm.io/gen. DO NOT EDIT.
// Code generated by gorm.io/gen. DO NOT EDIT.
// Code generated by gorm.io/gen. DO NOT EDIT.

package model

const TableNameLicense = "licenses"

// License mapped from table <licenses>
type License struct {
	ID          string `gorm:"column:id;primaryKey" json:"id"`
	Name        string `gorm:"column:name;not null" json:"name"`
	Description string `gorm:"column:description" json:"description"`
	Public      bool   `gorm:"column:public" json:"public"`
	Content     string `gorm:"column:content" json:"content"`
}

// TableName License's table name
func (*License) TableName() string {
	return TableNameLicense
}
