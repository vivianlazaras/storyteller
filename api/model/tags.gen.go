// Code generated by gorm.io/gen. DO NOT EDIT.
// Code generated by gorm.io/gen. DO NOT EDIT.
// Code generated by gorm.io/gen. DO NOT EDIT.

package model

const TableNameTag = "tags"

// Tag mapped from table <tags>
type Tag struct {
	ID     string `gorm:"column:id;primaryKey;default:gen_random_uuid()" json:"id"`
	Value  string `gorm:"column:value;not null" json:"value"`
	Entity string `gorm:"column:entity" json:"entity"`
}

// TableName Tag's table name
func (*Tag) TableName() string {
	return TableNameTag
}
