import os
import re

file1 = '.github/workflows/release-beta-on-push.yml'
with open(file1, 'r', encoding='utf-8') as f:
    text = f.read()

# 1. branches: [master] -> branches: [main, master]
text = text.replace('branches: [master]', 'branches: [main, master]')

# 2. remove upstream restrictions
text = text.replace("if: github.repository == 'zeroclaw-labs/zeroclaw' && needs.version.outputs.skip != 'true'", "if: needs.version.outputs.skip != 'true'")
text = text.replace("if: github.repository == 'zeroclaw-labs/zeroclaw'", "")

# 3. RELEASE_TOKEN -> GITHUB_TOKEN
text = text.replace('secrets.RELEASE_TOKEN', 'secrets.GITHUB_TOKEN')

# 4. Remove redeploy-website block
text = re.sub(r'\s+redeploy-website:.*?(?=\s+docker:|\Z)', '\n', text, flags=re.DOTALL)

with open(file1, 'w', encoding='utf-8') as f:
    f.write(text)


file2 = '.github/workflows/release-stable-manual.yml'
with open(file2, 'r', encoding='utf-8') as f:
    text2 = f.read()

# 1. RELEASE_TOKEN -> GITHUB_TOKEN
text2 = text2.replace('secrets.RELEASE_TOKEN', 'secrets.GITHUB_TOKEN')

# 2. Remove downstream jobs: crates-io to redeploy-website to docker. 
# Wait, docker comes AFTER redeploy-website. Let's just remove crates-io and redeploy-website.
text2 = re.sub(r'\s+crates-io:.*?(?=\s+docker:|\Z)', '\n', text2, flags=re.DOTALL)
# 3. Remove Post-publish (scoop, aur, homebrew, tweet, discord)
text2 = re.sub(r'\s+# ── Post-publish.*', '\n', text2, flags=re.DOTALL)

with open(file2, 'w', encoding='utf-8') as f:
    f.write(text2)

print('Updated both workflow files')
