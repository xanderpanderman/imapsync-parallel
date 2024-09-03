# hi

## I made this for me.

This utility can be used to migrate emails using imapsync from one host to another in bulk.

### It can be run like this:

`cargo run -- --csv-file-path ./TEST_CSV.csv --old-host 192.168.45.123 --new-host 10.0.15.250`

The csv should look like this:
| old_email | old_password | new_email | new_password |
|-----------------------|--------------|-----------------------|--------------|
| user1@oldhost.com | password1 | user1@newhost.com | password1 |
| user2@oldhost.com | password2 | user2@newhost.com | password2 |
| user3@oldhost.com | password3 | user3@newhost.com | password3 |
| user4@oldhost.com | password4 | user4@newhost.com | password4 |
