#!/usr/bin/env bash
#
# Folio Installation Test Script
#
# This script tests the installation process in an isolated environment.
# It can be run in CI or locally to verify the install script works.
#
# Usage:
#   ./scripts/test-install.sh           # Run all tests
#   ./scripts/test-install.sh --quick   # Skip build test (faster)
#

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
TEST_HOME="${TEST_HOME:-$(mktemp -d)}"
QUICK_MODE=false

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

# Test counters
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_SKIPPED=0

# Logging
info() {
    echo -e "${BLUE}[TEST]${NC} $1"
}

pass() {
    echo -e "${GREEN}[PASS]${NC} $1"
    TESTS_PASSED=$((TESTS_PASSED + 1))
}

fail() {
    echo -e "${RED}[FAIL]${NC} $1"
    TESTS_FAILED=$((TESTS_FAILED + 1))
}

skip() {
    echo -e "${YELLOW}[SKIP]${NC} $1"
    TESTS_SKIPPED=$((TESTS_SKIPPED + 1))
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --quick)
            QUICK_MODE=true
            shift
            ;;
        --test-home)
            TEST_HOME="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Cleanup function
cleanup() {
    info "Cleaning up test environment..."
    if [ -d "$TEST_HOME" ] && [[ "$TEST_HOME" == /tmp/* ]]; then
        rm -rf "$TEST_HOME"
    fi
}

trap cleanup EXIT

#
# Individual Tests
#

test_install_script_exists() {
    info "Testing: install.sh exists"
    if [ -f "$PROJECT_ROOT/install.sh" ]; then
        pass "install.sh exists"
    else
        fail "install.sh not found"
        return 1
    fi
}

test_install_script_executable() {
    info "Testing: install.sh is valid bash"
    if bash -n "$PROJECT_ROOT/install.sh" 2>/dev/null; then
        pass "install.sh has valid syntax"
    else
        fail "install.sh has syntax errors"
        return 1
    fi
}

test_install_script_help() {
    info "Testing: install.sh --help works"
    if bash "$PROJECT_ROOT/install.sh" --help 2>/dev/null | grep -q "USAGE"; then
        pass "install.sh --help shows usage"
    else
        fail "install.sh --help failed"
        return 1
    fi
}

test_cargo_build() {
    info "Testing: cargo build works"
    if [ "$QUICK_MODE" = true ]; then
        skip "cargo build (quick mode)"
        return 0
    fi

    cd "$PROJECT_ROOT"
    if cargo build --release --quiet 2>/dev/null; then
        pass "cargo build --release succeeded"
    else
        fail "cargo build --release failed"
        return 1
    fi
}

test_binary_exists() {
    info "Testing: binary exists after build"
    if [ "$QUICK_MODE" = true ]; then
        # In quick mode, check if binary already exists
        if [ -f "$PROJECT_ROOT/target/release/folio" ]; then
            pass "binary exists"
        else
            skip "binary check (not built)"
        fi
        return 0
    fi

    if [ -f "$PROJECT_ROOT/target/release/folio" ]; then
        pass "binary exists at target/release/folio"
    else
        fail "binary not found after build"
        return 1
    fi
}

test_binary_runs() {
    info "Testing: binary runs --version"
    local BINARY="$PROJECT_ROOT/target/release/folio"

    if [ ! -f "$BINARY" ]; then
        skip "binary --version (binary not built)"
        return 0
    fi

    if "$BINARY" --version 2>/dev/null | grep -q "folio"; then
        pass "binary --version works"
    else
        fail "binary --version failed"
        return 1
    fi
}

test_binary_help() {
    info "Testing: binary --help"
    local BINARY="$PROJECT_ROOT/target/release/folio"

    if [ ! -f "$BINARY" ]; then
        skip "binary --help (binary not built)"
        return 0
    fi

    if "$BINARY" --help 2>/dev/null | grep -q "capture"; then
        pass "binary --help shows commands"
    else
        fail "binary --help failed"
        return 1
    fi
}

test_simulated_install() {
    info "Testing: simulated installation to temp directory"
    local INSTALL_DIR="$TEST_HOME/.local/bin"
    local BINARY="$PROJECT_ROOT/target/release/folio"

    if [ ! -f "$BINARY" ]; then
        skip "simulated install (binary not built)"
        return 0
    fi

    mkdir -p "$INSTALL_DIR"
    cp "$BINARY" "$INSTALL_DIR/folio"
    chmod +x "$INSTALL_DIR/folio"

    if [ -x "$INSTALL_DIR/folio" ]; then
        pass "simulated install succeeded"
    else
        fail "simulated install failed"
        return 1
    fi
}

test_config_directory_creation() {
    info "Testing: config directory creation"
    local INSTALL_DIR="$TEST_HOME/.local/bin"
    local FOLIO_HOME="$TEST_HOME/.folio"

    if [ ! -x "$INSTALL_DIR/folio" ]; then
        skip "config directory (binary not installed)"
        return 0
    fi

    # Run a command that triggers config creation
    export HOME="$TEST_HOME"
    "$INSTALL_DIR/folio" stats 2>/dev/null || true

    if [ -d "$FOLIO_HOME" ]; then
        pass "config directory created at ~/.folio"
    else
        fail "config directory not created"
        return 1
    fi
}

test_database_creation() {
    info "Testing: database creation"
    local FOLIO_HOME="$TEST_HOME/.folio"

    if [ ! -d "$FOLIO_HOME" ]; then
        skip "database creation (config dir not created)"
        return 0
    fi

    if [ -f "$FOLIO_HOME/folio.db" ]; then
        pass "database file created"
    else
        fail "database file not created"
        return 1
    fi
}

test_capture_command() {
    info "Testing: capture command"
    local INSTALL_DIR="$TEST_HOME/.local/bin"

    if [ ! -x "$INSTALL_DIR/folio" ]; then
        skip "capture command (binary not installed)"
        return 0
    fi

    export HOME="$TEST_HOME"
    if "$INSTALL_DIR/folio" capture "Test accomplishment" --importance low 2>/dev/null; then
        pass "capture command works"
    else
        fail "capture command failed"
        return 1
    fi
}

test_list_command() {
    info "Testing: list command"
    local INSTALL_DIR="$TEST_HOME/.local/bin"

    if [ ! -x "$INSTALL_DIR/folio" ]; then
        skip "list command (binary not installed)"
        return 0
    fi

    export HOME="$TEST_HOME"
    if "$INSTALL_DIR/folio" list 2>/dev/null | grep -q "Test accomplishment"; then
        pass "list command shows captured item"
    else
        fail "list command failed to show captured item"
        return 1
    fi
}

test_stats_command() {
    info "Testing: stats command"
    local INSTALL_DIR="$TEST_HOME/.local/bin"

    if [ ! -x "$INSTALL_DIR/folio" ]; then
        skip "stats command (binary not installed)"
        return 0
    fi

    export HOME="$TEST_HOME"
    if "$INSTALL_DIR/folio" stats 2>/dev/null; then
        pass "stats command works"
    else
        fail "stats command failed"
        return 1
    fi
}

test_export_command() {
    info "Testing: export command"
    local INSTALL_DIR="$TEST_HOME/.local/bin"

    if [ ! -x "$INSTALL_DIR/folio" ]; then
        skip "export command (binary not installed)"
        return 0
    fi

    export HOME="$TEST_HOME"
    if "$INSTALL_DIR/folio" export --format json 2>/dev/null | grep -q "Test accomplishment"; then
        pass "export command works"
    else
        fail "export command failed"
        return 1
    fi
}

test_uninstall_flag() {
    info "Testing: install.sh --uninstall flag parsing"
    # Just test that the flag is recognized
    if bash "$PROJECT_ROOT/install.sh" --help 2>/dev/null | grep -q "uninstall"; then
        pass "uninstall flag documented"
    else
        fail "uninstall flag not documented"
        return 1
    fi
}

#
# Main test runner
#

print_header() {
    echo ""
    echo "╔════════════════════════════════════════════╗"
    echo "║     Folio Installation Test Suite          ║"
    echo "╚════════════════════════════════════════════╝"
    echo ""
    echo "Test environment: $TEST_HOME"
    echo "Quick mode: $QUICK_MODE"
    echo ""
}

print_summary() {
    echo ""
    echo "════════════════════════════════════════════"
    echo "Test Results"
    echo "════════════════════════════════════════════"
    echo -e "  ${GREEN}Passed:${NC}  $TESTS_PASSED"
    echo -e "  ${RED}Failed:${NC}  $TESTS_FAILED"
    echo -e "  ${YELLOW}Skipped:${NC} $TESTS_SKIPPED"
    echo "════════════════════════════════════════════"
    echo ""

    if [ $TESTS_FAILED -eq 0 ]; then
        echo -e "${GREEN}${BOLD}All tests passed!${NC}"
        return 0
    else
        echo -e "${RED}${BOLD}Some tests failed.${NC}"
        return 1
    fi
}

run_tests() {
    print_header

    # Install script tests
    test_install_script_exists
    test_install_script_executable
    test_install_script_help
    test_uninstall_flag

    # Build tests
    test_cargo_build
    test_binary_exists
    test_binary_runs
    test_binary_help

    # Installation simulation tests
    test_simulated_install
    test_config_directory_creation
    test_database_creation

    # Functional tests
    test_capture_command
    test_list_command
    test_stats_command
    test_export_command

    print_summary
}

# Run tests
run_tests
