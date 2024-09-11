# prevents local build artifacts being added to image
cd calcite-rs-jni
mvn clean
cd calcite
./gradlew clean

 create a tag name from the last connector release
cd ../..
release_info=$(curl -L \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer $GITHUB_TOKEN" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  https://api.github.com/repos/hasura/ndc-calcite/releases/latest)
TAG=$(echo "$release_info" | grep 'tag_name' | awk -F':' '{print $2}' | tr -d ' "",')

# build arm & amd versions
docker build . --no-cache --platform linux/arm64,linux/amd64 -t kstott/meta_connector:latest
#docker buildx build --platform linux/arm64 --output type=oci,dest=./image.tar .
docker tag kstott/meta_connector:latest kstott/meta_connector:"$TAG"

# push to docker hub
docker push kstott/meta_connector:latest
docker push kstott/meta_connector:"$TAG"
