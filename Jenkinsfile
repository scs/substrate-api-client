pipeline {
  agent {
    node {
      label 'rust&&sgx'
    }
  }
  options {
    timeout(time: 2, unit: 'HOURS')
    buildDiscarder(logRotator(numToKeepStr: '14'))
  }
  stages {
    stage('Environment') {
      steps {
        sh './ci/install_rust.sh'
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
    stage('Start substrate-test-nodes') {
      steps {
        copyArtifacts fingerprintArtifacts: true, projectName: 'substraTEE/substrate-test-nodes/master', selector: lastCompleted()
        sh 'target/release/substrate-test-node purge-chain --dev -y'
        sh 'target/release/substrate-test-node --dev &'
        sh 'sleep 10'
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
        sh 'target/release/examples/example_compose_extrinsic_offline'
        sh 'target/release/examples/example_custom_storage_struct'
        sh 'target/release/examples/example_generic_extrinsic'
        sh 'target/release/examples/example_print_metadata'
        sh 'target/release/examples/example_transfer'
        sh 'target/release/examples/example_get_storage'
        // echo 'Running tests which are known to hang (needs fixing)'
        // catchError(buildResult: 'SUCCESS', stageResult: 'FAILURE') {
        //   sh 'target/release/examples/example_contract'
        //   sh 'target/release/examples/example_event_callback'
        // }
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
        catchError(buildResult: 'SUCCESS', stageResult: 'FAILURE') {
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
        archiveArtifacts artifacts: '**/build_*.log', fingerprint: true
        archiveArtifacts artifacts: '**/clippy_*.log', fingerprint: true
      }
    }
  }
  post {
    changed {
        emailext (
          subject: "Jenkins Build '${env.JOB_NAME} [${env.BUILD_NUMBER}]' is ${currentBuild.currentResult}",
          body: "${env.JOB_NAME} build ${env.BUILD_NUMBER} changed state and is now ${currentBuild.currentResult}\n\nMore info at: ${env.BUILD_URL}",
          to: '${env.RECIPIENTS_SUBSTRATEE}'
        )
    }
  }
}
