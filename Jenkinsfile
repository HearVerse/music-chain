pipeline {
  agent any
  stages {
    stage('Checkout') {
      when {
        branch 'develop'
      }
      steps {
        checkout scm
      }
    }

    stage('Build Docker image') {
      when {
        branch 'develop'
      }
      steps {
        script {
          sh "docker build -t ${REGISTRY}/${IMAGE_NAME}:${TAG} ."
        }

      }
    }

    stage('Push Docker image') {
      when {
        branch 'develop'
      }
      steps {
        script {
          DOCKER_HUB_USERNAME = 'hearverse'
          DOCKER_HUB_PASSWORD = 'WAmhVda748bWeEs'

          // Log in to Docker Hub
          sh "echo '${DOCKER_HUB_PASSWORD}' | docker login -u '${DOCKER_HUB_USERNAME}' --password-stdin"

          // Push the Docker image
          sh "docker push ${REGISTRY}/${IMAGE_NAME}:${TAG}"

          // Log out from Docker Hub
          sh "docker logout"
        }

      }
    }

    stage('Deploy Substrate project') {
      when {
        branch 'develop'
      }
      steps {
        script {
          sh "docker run -it --name substrate-node -p 30333:30333 -p 9933:9933 -p 9944:9944 ${REGISTRY}/${IMAGE_NAME}:${TAG} --dev"
        }

      }
    }

  }
  environment {
    IMAGE_NAME = 'music_chain'
    REGISTRY = 'hearverse'
    TAG = "${env.BRANCH_NAME}_${env.BUILD_NUMBER}"
  }
}