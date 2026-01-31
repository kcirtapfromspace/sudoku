#!/usr/bin/env python3
"""
Appium tests for Sudoku celebrations using the DEBUG menu.
Long-press for 2 seconds to open the debug menu in the game.
"""
import time
from appium import webdriver
from appium.options.ios import XCUITestOptions
from appium.webdriver.common.appiumby import AppiumBy


def create_driver():
    """Create Appium driver connection."""
    options = XCUITestOptions()
    options.platform_name = "iOS"
    options.device_name = "iPhone 17"
    options.udid = "BDAD4971-B1A4-442E-B0D8-1EF35DDADD7F"
    options.bundle_id = "com.sudoku.app"
    options.automation_name = "XCUITest"
    options.no_reset = True

    driver = webdriver.Remote(
        command_executor="http://127.0.0.1:4723",
        options=options
    )
    driver.implicitly_wait(5)
    return driver


def go_to_main_menu(driver):
    """Navigate to main menu."""
    try:
        # Dismiss any dialogs
        try:
            cancel = driver.find_element(AppiumBy.ACCESSIBILITY_ID, "Cancel")
            cancel.click()
            time.sleep(0.3)
        except:
            pass

        try:
            resume = driver.find_element(AppiumBy.ACCESSIBILITY_ID, "Resume")
            resume.click()
            time.sleep(0.3)
        except:
            pass

        # Quit game if in one
        try:
            pause = driver.find_element(AppiumBy.ACCESSIBILITY_ID, "pause")
            pause.click()
            time.sleep(0.3)
            quit_btn = driver.find_element(AppiumBy.ACCESSIBILITY_ID, "Quit Game")
            quit_btn.click()
            time.sleep(0.3)
            quit_confirm = driver.find_element(AppiumBy.ACCESSIBILITY_ID, "Quit")
            quit_confirm.click()
            time.sleep(0.5)
        except:
            pass
    except:
        pass


def start_new_game(driver):
    """Start a new beginner game."""
    new_game = driver.find_element(AppiumBy.ACCESSIBILITY_ID, "New Game")
    new_game.click()
    time.sleep(0.5)
    beginner = driver.find_element(AppiumBy.ACCESSIBILITY_ID, "Beginner")
    beginner.click()
    time.sleep(1)


def open_debug_menu(driver):
    """Open debug menu via long press (2 seconds)."""
    print("  Opening debug menu (long press 2s)...")

    # Get screen size for center coordinates
    size = driver.get_window_size()
    center_x = size['width'] // 2
    center_y = size['height'] // 2

    # Perform long press using touch action
    driver.execute_script("mobile: touchAndHold", {
        "x": center_x,
        "y": center_y,
        "duration": 2.5  # 2.5 seconds to ensure it triggers
    })
    time.sleep(0.5)


def test_row_completion_celebration():
    """Test row completion celebration."""
    print("\n" + "=" * 60)
    print("TEST: Row Completion Celebration")
    print("=" * 60)

    driver = create_driver()
    try:
        go_to_main_menu(driver)
        time.sleep(0.3)
        start_new_game(driver)
        print("  Started new game")

        driver.save_screenshot("/tmp/debug_before_fill.png")

        # Open debug menu
        open_debug_menu(driver)

        driver.save_screenshot("/tmp/debug_menu_open.png")

        # Select "Fill Row 1 (except 1 cell)"
        try:
            fill_row = driver.find_element(AppiumBy.ACCESSIBILITY_ID, "Fill Row 1 (except 1 cell)")
            fill_row.click()
            time.sleep(0.5)
            print("  Filled row 1 except 1 cell")
        except Exception as e:
            print(f"  Could not find fill row option: {e}")
            # Try by partial text
            try:
                buttons = driver.find_elements(AppiumBy.CLASS_NAME, "XCUIElementTypeButton")
                for btn in buttons:
                    label = btn.get_attribute("label") or ""
                    if "Row" in label:
                        btn.click()
                        print(f"  Clicked: {label}")
                        time.sleep(0.5)
                        break
            except:
                pass

        driver.save_screenshot("/tmp/debug_row_filled.png")

        # Now find the empty cell in row 1 and complete it
        # Take screenshot to see the grid state
        time.sleep(0.3)
        driver.save_screenshot("/tmp/debug_before_complete.png")

        # Find empty cell and enter the correct number
        # We need to tap on the empty cell first
        print("  Looking for empty cell to complete row...")

        # Get all buttons that might be cells
        cells = driver.find_elements(AppiumBy.CLASS_NAME, "XCUIElementTypeButton")
        empty_cell = None
        for cell in cells:
            name = cell.get_attribute("name") or ""
            label = cell.get_attribute("label") or ""
            # Empty cells have " " as name
            if name == " " or label == " ":
                y_pos = int(cell.get_attribute("y") or 0)
                # Row 1 cells should be near top of grid (around y=89-130)
                if 80 < y_pos < 150:
                    empty_cell = cell
                    print(f"  Found empty cell at y={y_pos}")
                    break

        if empty_cell:
            empty_cell.click()
            time.sleep(0.3)
            print("  Tapped empty cell")

            # Try each number 1-9 to find the correct one
            for num in range(1, 10):
                try:
                    num_btn = driver.find_element(AppiumBy.ACCESSIBILITY_ID, str(num))
                    num_btn.click()
                    time.sleep(0.8)  # Wait for potential celebration

                    # Take screenshot to capture celebration
                    driver.save_screenshot(f"/tmp/debug_celebration_attempt_{num}.png")

                    # Check if celebration appeared
                    source = driver.page_source
                    if "Complete" in source or "🎉" in source:
                        print(f"  🎉 ROW CELEBRATION DETECTED after entering {num}!")
                        driver.save_screenshot("/tmp/debug_row_celebration.png")
                        print("\n✅ TEST PASSED: Row completion celebration works!")
                        return True
                except:
                    pass

        print("\n⚠️ TEST INCOMPLETE: Could not trigger row celebration")
        return False

    except Exception as e:
        print(f"\n❌ Error: {e}")
        driver.save_screenshot("/tmp/debug_error.png")
        return False
    finally:
        driver.quit()


def test_win_celebration():
    """Test win celebration by filling all but 1 cell."""
    print("\n" + "=" * 60)
    print("TEST: Win Celebration (Fill All Except 1)")
    print("=" * 60)

    driver = create_driver()
    try:
        go_to_main_menu(driver)
        time.sleep(0.3)
        start_new_game(driver)
        print("  Started new game")

        # Open debug menu
        open_debug_menu(driver)

        driver.save_screenshot("/tmp/debug_win_menu.png")

        # Select "Fill All (leave 1 cell) - Win Test"
        try:
            fill_all = driver.find_element(AppiumBy.ACCESSIBILITY_ID, "Fill All (leave 1 cell) - Win Test")
            fill_all.click()
            time.sleep(0.5)
            print("  Filled all except 1 cell")
        except Exception as e:
            print(f"  Looking for fill all option...")
            buttons = driver.find_elements(AppiumBy.CLASS_NAME, "XCUIElementTypeButton")
            for btn in buttons:
                label = btn.get_attribute("label") or ""
                if "Win Test" in label or "leave 1" in label:
                    btn.click()
                    print(f"  Clicked: {label}")
                    time.sleep(0.5)
                    break

        driver.save_screenshot("/tmp/debug_almost_complete.png")

        # Find and complete the last cell
        print("  Looking for the last empty cell...")
        cells = driver.find_elements(AppiumBy.CLASS_NAME, "XCUIElementTypeButton")
        empty_cell = None
        for cell in cells:
            name = cell.get_attribute("name") or ""
            if name == " ":
                empty_cell = cell
                break

        if empty_cell:
            empty_cell.click()
            time.sleep(0.3)
            print("  Tapped empty cell")

            # Try numbers to complete
            for num in range(1, 10):
                try:
                    num_btn = driver.find_element(AppiumBy.ACCESSIBILITY_ID, str(num))
                    num_btn.click()
                    time.sleep(1.5)  # Wait for win celebration

                    driver.save_screenshot(f"/tmp/debug_win_attempt_{num}.png")

                    # Check for win
                    source = driver.page_source
                    if "SOLVED" in source or "won" in source.lower() or "🏆" in source:
                        print(f"  🏆 WIN CELEBRATION DETECTED!")
                        driver.save_screenshot("/tmp/debug_win_celebration.png")
                        print("\n✅ TEST PASSED: Win celebration works!")
                        return True

                    # Check if we're back at menu (game ended)
                    try:
                        driver.find_element(AppiumBy.ACCESSIBILITY_ID, "New Game")
                        print("  Game ended - returned to main menu")
                        print("\n✅ TEST PASSED: Win detection triggered (game ended)")
                        return True
                    except:
                        pass
                except:
                    pass

        print("\n⚠️ TEST INCOMPLETE: Could not trigger win")
        return False

    except Exception as e:
        print(f"\n❌ Error: {e}")
        driver.save_screenshot("/tmp/debug_win_error.png")
        return False
    finally:
        driver.quit()


def run_all_tests():
    """Run all celebration tests."""
    print("\n" + "=" * 60)
    print("SUDOKU CELEBRATION TESTS (DEBUG MODE)")
    print("=" * 60)
    print("\nNOTE: These tests require DEBUG build with debug menu.")
    print("Long-press the game screen for 2s to open debug menu.\n")

    results = {}
    results["Row Completion"] = test_row_completion_celebration()
    results["Win Celebration"] = test_win_celebration()

    print("\n" + "=" * 60)
    print("TEST RESULTS")
    print("=" * 60)
    for name, passed in results.items():
        status = "✅ PASS" if passed else "❌ FAIL"
        print(f"  {status}: {name}")

    print("\nScreenshots saved to /tmp/debug_*.png")
    return all(results.values())


if __name__ == "__main__":
    # Start Appium if not running
    import subprocess
    import os

    # Check if Appium is running
    try:
        import urllib.request
        urllib.request.urlopen("http://127.0.0.1:4723/status", timeout=2)
        print("Appium is running")
    except:
        print("Starting Appium server...")
        subprocess.Popen(["appium", "--port", "4723"],
                        stdout=open("/tmp/appium.log", "w"),
                        stderr=subprocess.STDOUT)
        time.sleep(5)

    run_all_tests()
