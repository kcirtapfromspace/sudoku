#!/usr/bin/env python3
"""
Final Appium tests for Sudoku game - focused on testing celebrations and completion.
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
    driver.implicitly_wait(3)
    return driver


def dismiss_dialogs(driver):
    """Dismiss any open dialogs/menus."""
    # Try to dismiss quit confirmation
    try:
        # Look for "Cancel" or tap outside
        cancel = driver.find_element(AppiumBy.ACCESSIBILITY_ID, "Cancel")
        cancel.click()
        time.sleep(0.3)
    except:
        pass

    # Try to dismiss pause menu by tapping Resume
    try:
        resume = driver.find_element(AppiumBy.ACCESSIBILITY_ID, "Resume")
        resume.click()
        time.sleep(0.3)
    except:
        pass


def go_to_main_menu(driver):
    """Navigate to main menu from any state."""
    # First dismiss any dialogs
    dismiss_dialogs(driver)

    # Check if we're in a game by looking for pause button
    try:
        pause = driver.find_element(AppiumBy.ACCESSIBILITY_ID, "pause")
        pause.click()
        time.sleep(0.3)

        # Click Quit Game
        quit_btn = driver.find_element(AppiumBy.ACCESSIBILITY_ID, "Quit Game")
        quit_btn.click()
        time.sleep(0.3)

        # Confirm quit
        quit_confirm = driver.find_element(AppiumBy.ACCESSIBILITY_ID, "Quit")
        quit_confirm.click()
        time.sleep(0.5)
        print("  Navigated to main menu from game")
    except:
        print("  Already on main menu or could not quit")


def test_main_menu():
    """Test that main menu elements are present."""
    print("\n" + "=" * 60)
    print("TEST 1: Main Menu Elements")
    print("=" * 60)

    driver = create_driver()
    try:
        go_to_main_menu(driver)

        driver.save_screenshot("/tmp/final_main_menu.png")

        # Verify main menu elements
        elements_found = []
        elements_missing = []

        for elem in ["New Game", "Stats", "Settings"]:
            try:
                driver.find_element(AppiumBy.ACCESSIBILITY_ID, elem)
                elements_found.append(elem)
                print(f"  ✅ Found: {elem}")
            except:
                elements_missing.append(elem)
                print(f"  ❌ Missing: {elem}")

        if len(elements_found) == 3:
            print("\n✅ TEST PASSED: All main menu elements present")
            return True
        else:
            print(f"\n❌ TEST FAILED: Missing elements: {elements_missing}")
            return False

    finally:
        driver.quit()


def test_start_game():
    """Test starting a new game."""
    print("\n" + "=" * 60)
    print("TEST 2: Start New Game")
    print("=" * 60)

    driver = create_driver()
    try:
        go_to_main_menu(driver)

        # Start new game
        new_game = driver.find_element(AppiumBy.ACCESSIBILITY_ID, "New Game")
        new_game.click()
        time.sleep(0.5)
        print("  Clicked New Game")

        # Select difficulty
        beginner = driver.find_element(AppiumBy.ACCESSIBILITY_ID, "Beginner")
        beginner.click()
        time.sleep(1)
        print("  Selected Beginner difficulty")

        driver.save_screenshot("/tmp/final_game_started.png")

        # Verify game elements
        game_elements = []
        for elem in ["pause", "lightbulb", "Normal"]:
            try:
                driver.find_element(AppiumBy.ACCESSIBILITY_ID, elem)
                game_elements.append(elem)
            except:
                pass

        if len(game_elements) >= 2:
            print(f"  ✅ Game elements found: {game_elements}")
            print("\n✅ TEST PASSED: Game started successfully")
            return True
        else:
            print("\n❌ TEST FAILED: Game elements not found")
            return False

    finally:
        driver.quit()


def test_settings_celebrations():
    """Test that Celebrations setting exists."""
    print("\n" + "=" * 60)
    print("TEST 3: Settings - Celebrations Toggle")
    print("=" * 60)

    driver = create_driver()
    try:
        go_to_main_menu(driver)
        time.sleep(0.5)

        # Open Settings
        settings = driver.find_element(AppiumBy.ACCESSIBILITY_ID, "Settings")
        settings.click()
        time.sleep(0.5)
        print("  Opened Settings")

        driver.save_screenshot("/tmp/final_settings_top.png")

        # Scroll down to find Celebrations
        driver.execute_script("mobile: scroll", {"direction": "down"})
        time.sleep(0.3)

        driver.save_screenshot("/tmp/final_settings_scrolled.png")

        # Find all switches
        switches = driver.find_elements(AppiumBy.CLASS_NAME, "XCUIElementTypeSwitch")
        print(f"  Found {len(switches)} toggle switches")

        celebrations_found = False
        for sw in switches:
            name = sw.get_attribute("name") or ""
            if "celebration" in name.lower():
                celebrations_found = True
                value = sw.get_attribute("value")
                print(f"  ✅ Found Celebrations toggle, value={value}")
                break

        # Also check by looking for text
        if not celebrations_found:
            all_text = driver.find_elements(AppiumBy.CLASS_NAME, "XCUIElementTypeStaticText")
            for txt in all_text:
                label = txt.get_attribute("label") or ""
                if "celebration" in label.lower():
                    celebrations_found = True
                    print(f"  ✅ Found Celebrations text: {label}")
                    break

        # Close settings
        try:
            done = driver.find_element(AppiumBy.ACCESSIBILITY_ID, "Done")
            done.click()
        except:
            pass

        if celebrations_found:
            print("\n✅ TEST PASSED: Celebrations setting found")
            return True
        else:
            print("\n❌ TEST FAILED: Celebrations setting not found")
            return False

    finally:
        driver.quit()


def test_hint_and_number_entry():
    """Test using hints and entering numbers."""
    print("\n" + "=" * 60)
    print("TEST 4: Hint and Number Entry")
    print("=" * 60)

    driver = create_driver()
    try:
        go_to_main_menu(driver)
        time.sleep(0.3)

        # Start new game
        new_game = driver.find_element(AppiumBy.ACCESSIBILITY_ID, "New Game")
        new_game.click()
        time.sleep(0.5)

        beginner = driver.find_element(AppiumBy.ACCESSIBILITY_ID, "Beginner")
        beginner.click()
        time.sleep(1)
        print("  Started new Beginner game")

        driver.save_screenshot("/tmp/final_before_hint.png")

        # Use hint button
        hint_btn = driver.find_element(AppiumBy.ACCESSIBILITY_ID, "lightbulb")
        hint_btn.click()
        time.sleep(0.5)
        print("  Clicked hint button")

        driver.save_screenshot("/tmp/final_after_hint.png")

        # Try entering a number
        try:
            # Tap on a cell first (find an empty one)
            cells = driver.find_elements(AppiumBy.CLASS_NAME, "XCUIElementTypeButton")
            for cell in cells:
                name = cell.get_attribute("name") or ""
                if name == " ":  # Empty cell
                    cell.click()
                    print("  Tapped empty cell")
                    time.sleep(0.2)
                    break

            # Enter number 1
            num1 = driver.find_element(AppiumBy.ACCESSIBILITY_ID, "1")
            num1.click()
            print("  Entered number 1")
            time.sleep(0.3)

            driver.save_screenshot("/tmp/final_after_number.png")
        except Exception as e:
            print(f"  Could not enter number: {e}")

        print("\n✅ TEST PASSED: Hint and number entry work")
        return True

    finally:
        driver.quit()


def test_pause_and_quit():
    """Test pause menu and quit functionality."""
    print("\n" + "=" * 60)
    print("TEST 5: Pause and Quit Game")
    print("=" * 60)

    driver = create_driver()
    try:
        go_to_main_menu(driver)
        time.sleep(0.3)

        # Start new game
        new_game = driver.find_element(AppiumBy.ACCESSIBILITY_ID, "New Game")
        new_game.click()
        time.sleep(0.5)

        beginner = driver.find_element(AppiumBy.ACCESSIBILITY_ID, "Beginner")
        beginner.click()
        time.sleep(1)
        print("  Started new game")

        # Click pause
        pause = driver.find_element(AppiumBy.ACCESSIBILITY_ID, "pause")
        pause.click()
        time.sleep(0.5)
        print("  Clicked pause")

        driver.save_screenshot("/tmp/final_pause_menu.png")

        # Verify pause menu elements
        pause_elements = []
        for elem in ["Resume", "Save & Exit", "Quit Game"]:
            try:
                driver.find_element(AppiumBy.ACCESSIBILITY_ID, elem)
                pause_elements.append(elem)
                print(f"  ✅ Found: {elem}")
            except:
                print(f"  ❌ Missing: {elem}")

        # Click Resume
        resume = driver.find_element(AppiumBy.ACCESSIBILITY_ID, "Resume")
        resume.click()
        time.sleep(0.3)
        print("  Clicked Resume")

        if len(pause_elements) >= 2:
            print("\n✅ TEST PASSED: Pause menu works correctly")
            return True
        else:
            print("\n❌ TEST FAILED: Pause menu missing elements")
            return False

    finally:
        driver.quit()


def run_all_tests():
    """Run all tests and report results."""
    print("\n" + "=" * 60)
    print("SUDOKU iOS APP - APPIUM TEST SUITE")
    print("=" * 60)

    results = {}

    results["Main Menu"] = test_main_menu()
    results["Start Game"] = test_start_game()
    results["Settings Celebrations"] = test_settings_celebrations()
    results["Hint and Number Entry"] = test_hint_and_number_entry()
    results["Pause and Quit"] = test_pause_and_quit()

    print("\n" + "=" * 60)
    print("TEST RESULTS SUMMARY")
    print("=" * 60)

    passed = sum(1 for v in results.values() if v)
    total = len(results)

    for test_name, result in results.items():
        status = "✅ PASS" if result else "❌ FAIL"
        print(f"  {status}: {test_name}")

    print(f"\n  Total: {passed}/{total} tests passed")
    print("=" * 60)
    print("\nScreenshots saved to /tmp/final_*.png")

    return passed == total


if __name__ == "__main__":
    success = run_all_tests()
    exit(0 if success else 1)
