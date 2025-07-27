git fetch

echo -e "\033[1;32mNon-pushed commits:\033[0m"
git log origin/$(git branch --show-current)..$(git branch --show-current) --oneline

echo -e "\033[1;32mstatus\033[0m"
git status --short

echo -e "\033[1;32mStash:\033[0m"
git stash list

echo -e "\033[1;32mLocal branches without remote:\033[0m"
git branch -vv | grep ': gone]'