# Use this Makefile if you want a local image for testing purposes

REGION=eu-west-1
# Get your account id with aws ecr get-login --no-include-email --region $(REGION)
ACCOUNT_ID=449410177519
REGISTRY=$(ACCOUNT_ID).dkr.ecr.$(REGION).amazonaws.com

IMAGE_NAME=soda/test-service
IMAGE_VERSION=0.3.1

build:
	docker build -f Dockerfile -t $(IMAGE_NAME) .

tag: build
	docker tag $(IMAGE_NAME) $(IMAGE_NAME):$(IMAGE_VERSION)

tag-aws: build
	docker tag $(IMAGE_NAME) $(REGISTRY)/$(IMAGE_NAME):$(IMAGE_VERSION)

push: tag
	docker push $(REGISTRY)/$(IMAGE_NAME):$(IMAGE_VERSION)
