pipeline {
  agent {
    docker {
      image 'scssubstratee/substratee_dev:18.04-2.9.1-1.1.2'
      args '''
        -u root
        --privileged
      '''
    }
  }
  options {
    timeout(time: 2, unit: 'HOURS')
    buildDiscarder(logRotator(numToKeepStr: '3'))
  }
  stages {
    stage('rustup') {
      steps {
        sh './ci/install_rust.sh'
      }
    }
    stage('Start substrate-test-nodes') {
      steps {
        copyArtifacts fingerprintArtifacts: true, projectName: 'substraTEE/substrate-api-client-test-node_nightly', selector: lastCompleted()
        sh 'target/release/node-template purge-chain --dev -y'
        sh 'target/release/node-template --dev &'
      }
    }
    stage('Build') {
      steps {
        sh 'cargo build --message-format json > ${WORKSPACE}/build_debug.log'
        sh 'cargo build --release --message-format json > ${WORKSPACE}/build_release.log'
        sh 'cargo build --release --no-default-features --message-format json > ${WORKSPACE}/build_release_no_default_features.log'
      }
    }
    stage('Build no_std') {
      steps {
        catchError(buildResult: 'UNSTABLE', stageResult: 'FAILURE') {
          sh 'cd test_no_std && cargo build --message-format json > ${WORKSPACE}/build_no_std.log'
        }
      }
    }
    stage('Build examples') {
      steps {
        sh 'cargo build --examples --message-format json > ${WORKSPACE}/build_examples_debug.log'
        sh 'cargo build --release --examples --message-format json > ${WORKSPACE}/build_examples_release.log'
      }
    }
    stage('Unit tests') {
      options {
        timeout(time: 5, unit: 'MINUTES')
      }
      steps {
        catchError(buildResult: 'UNSTABLE', stageResult: 'FAILURE') {
          sh 'cargo test'
        }
      }
    }
    stage('Test against substrate-test-node') {
      options {
        timeout(time: 1, unit: 'MINUTES')
      }
      steps {
        // run examples
        sh 'target/release/examples/example_generic_extrinsic'
        sh 'target/release/examples/example_print_metadata'
        sh 'target/release/examples/example_transfer'
        sh 'target/release/examples/example_get_storage'
        sh 'target/release/examples/example_get_blocks'
        sh 'target/release/examples/example_benchmark_bulk_xt'
        // TODO: example needs fixing sh 'target/release/examples/example_sudo'
      }
    }
    stage('Clippy') {
      steps {
        sh 'cargo clean'
        catchError(buildResult: 'SUCCESS', stageResult: 'FAILURE') {
          sh 'cargo clippy --release 2>&1 | tee ${WORKSPACE}/clippy_release.log'
        }
        catchError(buildResult: 'SUCCESS', stageResult: 'FAILURE') {
          sh 'cargo clippy --release --no-default-features 2>&1 | tee ${WORKSPACE}/clippy_release_no_default_features.log'
        }
        catchError(buildResult: 'SUCCESS', stageResult: 'FAILURE') {
          sh 'cd test_no_std && cargo clippy 2>&1 | tee ${WORKSPACE}/clippy_no_std.log'
        }
        catchError(buildResult: 'SUCCESS', stageResult: 'FAILURE') {
          sh 'cargo clippy --release --examples 2>&1 | tee ${WORKSPACE}/clippy_examples_release.log'
        }
      }
    }
    stage('Formater') {
      steps {
        catchError(buildResult: 'SUCCESS', stageResult: 'UNSTABLE') {
          sh 'cargo fmt -- --check > ${WORKSPACE}/fmt.log'
        }
      }
    }
    stage('Results') {
      steps {
        recordIssues(
          aggregatingResults: true,
          enabledForFailure: true,
          qualityGates: [[threshold: 1, type: 'TOTAL', unstable: true]],
          tools: [
              cargo(
                pattern: 'build_*.log',
                reportEncoding: 'UTF-8'
              ),
              groovyScript(
                parserId:'clippy-warnings',
                pattern: 'clippy_*.log',
                reportEncoding: 'UTF-8'
              ),
              groovyScript(
                parserId:'clippy-errors',
                pattern: 'clippy_*.log',
                reportEncoding: 'UTF-8'
              )
          ]
        )
        script {
          try {
            sh './ci/check_fmt_log.sh'
          }
          catch (exc) {
            echo 'Style changes detected. Setting build to unstable'
            currentBuild.result = 'UNSTABLE'
          }
        }
      }
    }
    stage('Archive artifact') {
      steps {
        archiveArtifacts artifacts: '*.log', fingerprint: true
      }
    }
  }
  post {
    unsuccessful {
      emailext (
        subject: "Jenkins Build '${env.JOB_NAME} [${env.BUILD_NUMBER}]' is ${currentBuild.currentResult}",
        body: "${env.JOB_NAME} build ${env.BUILD_NUMBER} is ${currentBuild.currentResult}\n\nMore info at: ${env.BUILD_URL}",
        to: "${env.RECIPIENTS_SUBSTRATEE}"
      )
    }
    always {
      cleanWs()
    }
  }
}