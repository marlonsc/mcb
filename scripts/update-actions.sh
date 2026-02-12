#!/bin/bash
set -e

WORKFLOW_DIR=".github/workflows"
echo "Scanning workflows in $WORKFLOW_DIR for action updates..."

# Find all unique actions used in workflows
# Matches "uses: owner/repo@ref" and extracts "owner/repo"
# We handle the 'uses:' prefix and split by @
actions=$(grep -r "uses:" "$WORKFLOW_DIR" | grep -v "\./" | grep -v "docker://" | awk '{for(i=1;i<=NF;i++) if($i ~ /^uses:/) print $(i+1)}' | cut -d@ -f1 | sort -u)

for action in $actions; do
    # Cleanup action name (remove 'uses:' prefix if awk didn't catch it cleanly, though logic above should)
    action="${action#uses:}"
    
    # Skip if empty or starting with quote/comment
    if [[ -z "$action" || "$action" == \#* ]]; then continue; fi

    echo "Checking latest version for $action..."

    # Get latest release tag
    # We use --exclude-pre-releases to ensure stability
    if [[ "$action" == "actions/checkout" ]]; then
        # Force v4 for checkout as v6 seems anomalous/experimental
        latest_tag=$(gh release list --repo "$action" --limit 20 --exclude-drafts --exclude-pre-releases --json tagName --jq '.[] | select(.tagName | startswith("v4")) | .tagName' | head -n 1 2>/dev/null || true)
    else
        latest_tag=$(gh release list --repo "$action" --limit 1 --exclude-drafts --exclude-pre-releases --json tagName --jq '.[0].tagName' 2>/dev/null || true)
    fi

    # Some repos like codeql-action are monorepos and might return different tags or errors
    # If gh release list fails or returns empty, check for tags manually via api/tags if needed, 
    # but for standard actions this works.
    
    commit_sha=""
    comment=""

    if [[ -z "$latest_tag" ]]; then
        # Fallback: get latest commit SHA from HEAD (unsafe but necessary if no releases)
        echo "  -> No release found. Checking HEAD..."
        commit_sha=$(gh api "repos/$action/commits/HEAD" --jq '.sha' 2>/dev/null || true)
        if [[ -z "$commit_sha" ]]; then
            echo "  -> Could not determine version for $action. Skipping."
            continue
        fi
        echo "  -> Using HEAD SHA: $commit_sha"
    else
        # Get SHA for the tag
        # We try accessing the commit associated with the tag
        # Note: annotated tags vs lightweight tags have different API responses.
        # "git/ref/tags/TAG" gives the object. If object.type is commit, use it. If tag, peel it.
        # Simpler: use the generic /commits/REF endpoint
        commit_sha=$(gh api "repos/$action/commits/$latest_tag" --jq '.sha' 2>/dev/null || true)
        
        if [[ -z "$commit_sha" ]]; then
             echo "  -> Could not resolve SHA for tag $latest_tag. Skipping."
             continue
        fi
        
        echo "  -> Found release: $latest_tag ($commit_sha)"
        comment=" # $latest_tag"
    fi

    # Escape for sed
    # We want to replace "uses: action@..." with "uses: action@sha # tag"
    # We need to catch the existing line content to avoid breaking indentation if possible, 
    # but strictly matching "uses: action@.*" works fine for YAML.
    
    # Pattern: "uses: action@..." -> "uses: action@SHA # tag"
    # We rely on the fact that 'uses:' is usually the start of the value.
    # We limit replacement to lines containing exact match.
    
    # Using find + xargs to handle file list
    for file in "$WORKFLOW_DIR"/*.yml; do
        # Use temp file for sed compliance
        # We want to match: (spaces)uses: action@something(eol or comment)
        # And replace with: (spaces)uses: action@sha # tag
        
        # Determine strict regex for the action
        # - Uses @ as delimiter
        # - We want to replace everything after @ until end of line or before comment?
        # Simpler: just replace the whole reference.
        
        # Note: sed syntax varies. We use | as delimiter.
        # We escape the action name just in case.
        esc_action=$(echo "$action" | sed 's/\//\\\//g')
        
        sed -i "s|uses: $esc_action@.*|uses: $action@$commit_sha$comment|g" "$file"
    done
done

echo "Update complete."
