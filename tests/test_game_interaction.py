#!/usr/bin/env python3
"""
Appium tests for Sudoku game - cell interaction and celebrations.
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


def test_cell_interaction():
    """Test tapping cells and entering numbers."""
    print("\n" + "=" * 60)
    print("TEST: Cell Interaction")
    print("=" * 60)

    driver = create_driver()

    try:
        # Take initial screenshot
        driver.save_screenshot("/tmp/test_initial.png")
        print("Initial screenshot saved")

        # Check if we're on main menu or in game
        try:
            new_game = driver.find_element(AppiumBy.ACCESSIBILITY_ID, "New Game")
            print("On main menu - starting new game...")
            new_game.click()
            time.sleep(0.5)

            beginner = driver.find_element(AppiumBy.ACCESSIBILITY_ID, "Beginner")
            beginner.click()
            time.sleep(1)
            print("Started Beginner game")
        except:
            print("Already in a game")

        # Get page source to analyze structure
        source = driver.page_source
        with open("/tmp/game_source.xml", "w") as f:
            f.write(source)
        print("Page source saved to /tmp/game_source.xml")

        # Find all cells (they appear as buttons with single digit or space)
        all_buttons = driver.find_elements(AppiumBy.CLASS_NAME, "XCUIElementTypeButton")
        print(f"Found {len(all_buttons)} buttons total")

        # Find empty cells (labeled with space " ")
        empty_cells = []
        for btn in all_buttons:
            try:
                label = btn.get_attribute("label")
                name = btn.get_attribute("name")
                if label == " " or name == " ":
                    x = btn.get_attribute("x")
                    y = btn.get_attribute("y")
                    empty_cells.append({"element": btn, "x": x, "y": y})
            except:
                pass

        print(f"Found {len(empty_cells)} empty cells")

        if empty_cells:
            # Tap the first empty cell
            first_empty = empty_cells[0]["element"]
            first_empty.click()
            print("Tapped first empty cell")
            time.sleep(0.3)

            driver.save_screenshot("/tmp/test_cell_selected.png")
            print("Screenshot after cell selection saved")

            # Try to enter a number using the number pad
            try:
                # Find number button "1"
                num_btn = driver.find_element(AppiumBy.ACCESSIBILITY_ID, "1")
                num_btn.click()
                print("Entered number 1")
                time.sleep(0.5)

                driver.save_screenshot("/tmp/test_number_entered.png")
                print("Screenshot after number entry saved")
            except Exception as e:
                print(f"Could not enter number: {e}")

        # Take final screenshot
        driver.save_screenshot("/tmp/test_final.png")
        print("Final screenshot saved")

        print("\n✅ Cell interaction test completed!")

    except Exception as e:
        print(f"\n❌ Error: {e}")
        driver.save_screenshot("/tmp/test_error.png")
    finally:
        driver.quit()


def test_settings_celebrations():
    """Test the celebrations toggle in settings."""
    print("\n" + "=" * 60)
    print("TEST: Settings - Celebrations Toggle")
    print("=" * 60)

    driver = create_driver()

    try:
        # First go back to main menu if in game
        try:
            # Try to find pause button
            pause = driver.find_element(AppiumBy.ACCESSIBILITY_ID, "pause")
            pause.click()
            time.sleep(0.5)

            # Find Quit Game
            quit_btn = driver.find_element(AppiumBy.ACCESSIBILITY_ID, "Quit Game")
            quit_btn.click()
            time.sleep(0.5)
            print("Quit game, now on main menu")
        except:
            print("Already on main menu or pause menu not found")

        # Take screenshot
        driver.save_screenshot("/tmp/test_menu.png")

        # Open Settings
        settings = driver.find_element(AppiumBy.ACCESSIBILITY_ID, "Settings")
        settings.click()
        time.sleep(0.5)
        print("Opened Settings")

        driver.save_screenshot("/tmp/test_settings_top.png")

        # Scroll down to find Celebrations
        driver.execute_script("mobile: scroll", {"direction": "down"})
        time.sleep(0.3)

        driver.save_screenshot("/tmp/test_settings_scrolled.png")

        # Find and verify Celebrations toggle
        try:
            # Look for the switch associated with Celebrations
            celebrations = driver.find_element(AppiumBy.ACCESSIBILITY_ID, "Celebrations")
            print("✅ Found Celebrations setting!")

            # Get current value
            value = celebrations.get_attribute("value")
            print(f"Celebrations toggle value: {value}")

        except Exception as e:
            print(f"Could not find Celebrations by accessibility ID: {e}")

            # Try to find by looking for text
            all_text = driver.find_elements(AppiumBy.CLASS_NAME, "XCUIElementTypeStaticText")
            for txt in all_text:
                try:
                    label = txt.get_attribute("label")
                    if "celebration" in label.lower():
                        print(f"Found text: {label}")
                except:
                    pass

            # Try to find switches
            switches = driver.find_elements(AppiumBy.CLASS_NAME, "XCUIElementTypeSwitch")
            print(f"Found {len(switches)} switches")
            for i, sw in enumerate(switches):
                try:
                    name = sw.get_attribute("name")
                    value = sw.get_attribute("value")
                    print(f"  Switch {i}: name='{name}', value='{value}'")
                except:
                    pass

        # Close settings
        try:
            done = driver.find_element(AppiumBy.ACCESSIBILITY_ID, "Done")
            done.click()
            print("Closed Settings")
        except:
            pass

        print("\n✅ Settings test completed!")

    except Exception as e:
        print(f"\n❌ Error: {e}")
        driver.save_screenshot("/tmp/test_settings_error.png")
    finally:
        driver.quit()


def test_game_completion_flow():
    """Test game completion and win detection."""
    print("\n" + "=" * 60)
    print("TEST: Game Completion Flow")
    print("=" * 60)

    driver = create_driver()

    try:
        # Start fresh - go to main menu first
        try:
            pause = driver.find_element(AppiumBy.ACCESSIBILITY_ID, "pause")
            pause.click()
            time.sleep(0.5)
            quit_btn = driver.find_element(AppiumBy.ACCESSIBILITY_ID, "Quit Game")
            quit_btn.click()
            time.sleep(0.5)
        except:
            pass

        # Check current screen
        driver.save_screenshot("/tmp/test_completion_start.png")

        # Start new game
        try:
            new_game = driver.find_element(AppiumBy.ACCESSIBILITY_ID, "New Game")
            new_game.click()
            time.sleep(0.5)
            beginner = driver.find_element(AppiumBy.ACCESSIBILITY_ID, "Beginner")
            beginner.click()
            time.sleep(1)
            print("Started new Beginner game")
        except:
            print("Could not start new game")

        driver.save_screenshot("/tmp/test_completion_game.png")

        # Find the hint button and use it multiple times to auto-solve cells
        print("\nUsing hints to progress the game...")
        hint_count = 0
        max_hints = 50  # Limit hints to avoid infinite loop

        while hint_count < max_hints:
            try:
                hint_btn = driver.find_element(AppiumBy.ACCESSIBILITY_ID, "lightbulb")
                hint_btn.click()
                hint_count += 1
                time.sleep(0.3)

                # Take screenshot every 10 hints to see progress
                if hint_count % 10 == 0:
                    driver.save_screenshot(f"/tmp/test_hint_{hint_count}.png")
                    print(f"  Used {hint_count} hints...")

                # Check if we see a celebration or completion
                try:
                    # Look for celebration overlay or win screen
                    source = driver.page_source
                    if "Complete" in source or "SOLVED" in source or "🎉" in source:
                        print("Detected completion/celebration text!")
                        driver.save_screenshot(f"/tmp/test_celebration_{hint_count}.png")
                except:
                    pass

            except Exception as e:
                print(f"Hint button not found or error: {e}")
                break

        print(f"Used {hint_count} hints total")
        driver.save_screenshot("/tmp/test_completion_final.png")

        # Check final state
        source = driver.page_source
        if "won" in source.lower() or "complete" in source.lower():
            print("✅ Game completion detected!")
        else:
            print("Game may not be complete yet")

        print("\n✅ Game completion flow test completed!")

    except Exception as e:
        print(f"\n❌ Error: {e}")
        driver.save_screenshot("/tmp/test_completion_error.png")
    finally:
        driver.quit()


def run_all_tests():
    """Run all tests."""
    print("\n" + "=" * 60)
    print("SUDOKU GAME APPIUM TESTS")
    print("=" * 60)

    test_cell_interaction()
    test_settings_celebrations()
    test_game_completion_flow()

    print("\n" + "=" * 60)
    print("ALL TESTS COMPLETED")
    print("=" * 60)
    print("\nScreenshots saved to /tmp/test_*.png")


if __name__ == "__main__":
    run_all_tests()
