SELECT usergroups.id
FROM usergroups
LEFT JOIN grouprel ON usergroups.id = grouprel.groupid
WHERE usergroups.name = 'null'
GROUP BY usergroups.id
HAVING COUNT(grouprel.userid) = 0;