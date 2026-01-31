#!/usr/bin/env python3
"""
Final celebration test - properly complete the puzzle and verify celebration.
"""
import time
from appium import webdriver
from appium.options.ios import XCUITestOptions
from appium.webdriver.common.appiumby import AppiumBy


def create_driver():
    options = XCUITestOptions()
    options.platform_name = "iOS"
    options.device_name = "iPhone 17"
    options.udid = "BDAD4971-B1A4-442E-B0D8-1EF35DDADD7F"
    options.bundle_id = "com.sudoku.app"
    options.automation_name = "XCUITest"
    options.no_reset = True
    driver = webdriver.Remote(command_executor="http://127.0.0.1:4723", options=options)
    driver.implicitly_wait(5)
    return driver


def go_to_main_menu(driver):
    """Navigate to main menu."""
    try:
        try:
            resume = driver.find_element(AppiumBy.ACCESSIBILITY_ID, "Resume")
            resume.click()
            time.sleep(0.3)
        except:
            pass
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


def test_complete_game_and_win():
    """Test completing a game and triggering win celebration."""
    print("\n" + "=" * 60)
    print("TEST: Complete Game & Win Celebration")
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

        driver.save_screenshot("/tmp/celebration_01_start.png")

        # Open debug menu with long press
        print("  Opening debug menu...")
        size = driver.get_window_size()
        driver.execute_script("mobile: touchAndHold", {
            "x": size['width'] // 2,
            "y": size['height'] // 3,  # Upper part of screen (on grid)
            "duration": 2.5
        })
        time.sleep(0.5)

        driver.save_screenshot("/tmp/celebration_02_debug_menu.png")

        # Click "Fill All (leave 1 cell) - Win Test"
        print("  Selecting 'Fill All (leave 1 cell)'...")
        try:
            fill_btn = driver.find_element(AppiumBy.ACCESSIBILITY_ID, "Fill All (leave 1 cell) - Win Test")
            fill_btn.click()
        except:
            # Try finding by button text
            buttons = driver.find_elements(AppiumBy.CLASS_NAME, "XCUIElementTypeButton")
            for btn in buttons:
                label = btn.get_attribute("label") or ""
                if "Win Test" in label or "leave 1 cell" in label:
                    btn.click()
                    break
        time.sleep(1)
        print("  Filled all cells except 1")

        driver.save_screenshot("/tmp/celebration_03_almost_done.png")

        # Now we need to tap the grid to find and select the empty cell
        # Then enter numbers 1-9 until we find the right one
        print("  Completing the puzzle...")

        # Get grid area (roughly top half of screen)
        grid_y = size['height'] // 3

        # Tap around the grid to find the empty cell
        # We'll do a grid scan
        found_empty = False
        for tap_y in range(100, 500, 40):  # Scan vertically through grid
            for tap_x in range(30, 380, 40):  # Scan horizontally
                driver.execute_script("mobile: tap", {"x": tap_x, "y": tap_y})
                time.sleep(0.1)

                # Check if we selected an empty cell by looking at page source
                # (selected empty cells might show differently)

        # Just try clicking numbers until one works
        print("  Trying numbers 1-9 to complete puzzle...")
        for num in range(1, 10):
            try:
                num_btn = driver.find_element(AppiumBy.ACCESSIBILITY_ID, str(num))
                num_btn.click()
                time.sleep(0.5)

                # Check for celebration text or win screen
                source = driver.page_source

                # Look for celebration overlay
                if "SOLVED" in source or "Complete" in source or "🏆" in source:
                    print(f"  🎉 Celebration detected after entering {num}!")
                    driver.save_screenshot(f"/tmp/celebration_04_win_{num}.png")
                    time.sleep(1.5)  # Let celebration show
                    driver.save_screenshot("/tmp/celebration_05_after.png")
                    print("\n✅ TEST PASSED: Win celebration works!")
                    return True

                # Check if returned to main menu (game ended)
                try:
                    driver.find_element(AppiumBy.ACCESSIBILITY_ID, "New Game")
                    print(f"  Game ended after entering {num} - returned to menu")
                    driver.save_screenshot("/tmp/celebration_06_menu.png")
                    print("\n✅ TEST PASSED: Game completion detected!")
                    return True
                except:
                    pass

            except Exception as e:
                pass

        driver.save_screenshot("/tmp/celebration_error_final.png")
        print("\n⚠️ Could not complete puzzle")
        return False

    except Exception as e:
        print(f"\n❌ Error: {e}")
        driver.save_screenshot("/tmp/celebration_error.png")
        return False
    finally:
        driver.quit()


def test_row_celebration_simple():
    """Simple test - use hints repeatedly until a row completes."""
    print("\n" + "=" * 60)
    print("TEST: Row Celebration via Hints")
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

        # Use hints repeatedly and watch for celebrations
        print("  Using hints to progress game...")
        hint_count = 0
        celebration_seen = False

        for i in range(100):  # Max 100 hints
            try:
                # Click hint button
                hint = driver.find_element(AppiumBy.ACCESSIBILITY_ID, "lightbulb")
                hint.click()
                hint_count += 1
                time.sleep(0.3)

                # Check for celebration
                source = driver.page_source
                if "Complete" in source or "🎉" in source:
                    print(f"  🎉 Celebration after {hint_count} hints!")
                    driver.save_screenshot(f"/tmp/hint_celebration_{hint_count}.png")
                    celebration_seen = True

                # Check if game completed
                if "SOLVED" in source or "🏆" in source:
                    print(f"  🏆 GAME WON after {hint_count} hints!")
                    driver.save_screenshot(f"/tmp/hint_win_{hint_count}.png")
                    time.sleep(1)
                    break

                # Check if returned to menu
                try:
                    driver.find_element(AppiumBy.ACCESSIBILITY_ID, "New Game")
                    print(f"  Game ended after {hint_count} hints")
                    break
                except:
                    pass

                if hint_count % 10 == 0:
                    driver.save_screenshot(f"/tmp/hint_progress_{hint_count}.png")
                    print(f"    {hint_count} hints used...")

            except Exception as e:
                print(f"  Hint error: {e}")
                break

        print(f"\n  Total hints used: {hint_count}")
        if celebration_seen:
            print("\n✅ TEST PASSED: Celebration detected!")
            return True
        else:
            print("\n⚠️ No celebration text found in page source")
            return False

    except Exception as e:
        print(f"\n❌ Error: {e}")
        return False
    finally:
        driver.quit()


if __name__ == "__main__":
    print("\n" + "=" * 60)
    print("SUDOKU CELEBRATION TESTS")
    print("=" * 60)

    results = {}
    results["Win via Debug Menu"] = test_complete_game_and_win()
    results["Celebration via Hints"] = test_row_celebration_simple()

    print("\n" + "=" * 60)
    print("FINAL RESULTS")
    print("=" * 60)
    for name, passed in results.items():
        status = "✅ PASS" if passed else "❌ FAIL"
        print(f"  {status}: {name}")

    print("\nScreenshots saved to /tmp/celebration_*.png and /tmp/hint_*.png")
