# Variables
BRANCH_MAIN = main
BRANCH_DEPLOY = gh-pages
BUILD_DIR = target/dx/moonlight/release/web/public
TARGET_DIR = target

.PHONY: build deploy return

# Build the project
build:
	dx build --release

# Deploy to gh-pages
deploy: build
	git stash # Stash changes, including the built `target` directory
	git checkout $(BRANCH_DEPLOY)
	git stash pop # Restore the stashed `target` directory
	cp -r $(BUILD_DIR)/* ./
	rm -rf $(TARGET_DIR)
	git add --all
	git commit -m "deploy"
	git push
	make return

# Return to the main branch
return:
	git checkout $(BRANCH_MAIN)

