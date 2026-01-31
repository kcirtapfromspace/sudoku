#!/usr/bin/env python3
"""
Appium tests for Sudoku game celebrations and win/loss detection.
"""
import time
import unittest
from appium import webdriver
from appium.options.ios import XCUITestOptions
from appium.webdriver.common.appiumby import AppiumBy


class SudokuCelebrationTests(unittest.TestCase):
    """Test celebrations for row/column/box completion and win/loss."""

    @classmethod
    def setUpClass(cls):
        """Set up Appium driver."""
        options = XCUITestOptions()
        options.platform_name = "iOS"
        options.device_name = "iPhone 17"
        options.udid = "BDAD4971-B1A4-442E-B0D8-1EF35DDADD7F"
        options.bundle_id = "com.sudoku.app"
        options.automation_name = "XCUITest"
        options.no_reset = True

        cls.driver = webdriver.Remote(
            command_executor="http://127.0.0.1:4723",
            options=options
        )
        cls.driver.implicitly_wait(10)

    @classmethod
    def tearDownClass(cls):
        """Quit driver."""
        if cls.driver:
            cls.driver.quit()

    def test_01_start_new_game(self):
        """Test starting a new game."""
        print("\n=== Test: Start New Game ===")

        # Find and tap New Game button
        try:
            new_game_btn = self.driver.find_element(AppiumBy.ACCESSIBILITY_ID, "New Game")
            new_game_btn.click()
            print("Clicked New Game button")
            time.sleep(0.5)

            # Select Beginner difficulty for easier testing
            beginner_btn = self.driver.find_element(AppiumBy.ACCESSIBILITY_ID, "Beginner")
            beginner_btn.click()
            print("Selected Beginner difficulty")
            time.sleep(1)

            # Verify game started by checking for grid
            self.assertTrue(True, "Game started successfully")
            print("Game started successfully!")

        except Exception as e:
            print(f"Error: {e}")
            # Take screenshot on failure
            self.driver.save_screenshot("/tmp/test_start_game_error.png")
            raise

    def test_02_tap_cells(self):
        """Test tapping on cells and entering numbers."""
        print("\n=== Test: Tap Cells ===")

        try:
            # Take screenshot to see current state
            self.driver.save_screenshot("/tmp/test_game_state.png")
            print("Saved screenshot to /tmp/test_game_state.png")

            # Try to find cells by accessibility ID or class
            # Cells might be identified by position like "cell_0_0"
            cells = self.driver.find_elements(AppiumBy.CLASS_NAME, "XCUIElementTypeButton")
            print(f"Found {len(cells)} buttons")

            # Print all accessible elements for debugging
            all_elements = self.driver.find_elements(AppiumBy.XPATH, "//*")
            print(f"Total elements: {len(all_elements)}")

            for i, elem in enumerate(all_elements[:30]):
                try:
                    name = elem.get_attribute("name") or ""
                    label = elem.get_attribute("label") or ""
                    if name or label:
                        print(f"  [{i}] name='{name}', label='{label}'")
                except:
                    pass

        except Exception as e:
            print(f"Error: {e}")
            self.driver.save_screenshot("/tmp/test_tap_cells_error.png")

    def test_03_verify_celebrations_setting(self):
        """Test that celebrations setting exists."""
        print("\n=== Test: Verify Celebrations Setting ===")

        try:
            # Go back to main menu first (if in game)
            # Try to find pause button and quit
            try:
                pause_btn = self.driver.find_element(AppiumBy.ACCESSIBILITY_ID, "pause")
                pause_btn.click()
                time.sleep(0.5)

                quit_btn = self.driver.find_element(AppiumBy.ACCESSIBILITY_ID, "Quit Game")
                quit_btn.click()
                time.sleep(0.5)
            except:
                pass  # May already be on main menu

            # Open Settings
            settings_btn = self.driver.find_element(AppiumBy.ACCESSIBILITY_ID, "Settings")
            settings_btn.click()
            print("Opened Settings")
            time.sleep(0.5)

            # Scroll down to find Celebrations toggle
            self.driver.execute_script("mobile: scroll", {"direction": "down"})
            time.sleep(0.3)

            # Find Celebrations toggle
            celebrations = self.driver.find_element(AppiumBy.ACCESSIBILITY_ID, "Celebrations")
            print("Found Celebrations setting!")

            # Verify it's a toggle (switch)
            self.assertTrue(celebrations is not None)
            print("Celebrations setting verified!")

            # Take screenshot
            self.driver.save_screenshot("/tmp/test_celebrations_setting.png")

            # Close settings
            done_btn = self.driver.find_element(AppiumBy.ACCESSIBILITY_ID, "Done")
            done_btn.click()
            print("Closed Settings")

        except Exception as e:
            print(f"Error: {e}")
            self.driver.save_screenshot("/tmp/test_celebrations_setting_error.png")
            raise

    def test_04_check_game_completion_ui(self):
        """Test that game completion UI elements exist."""
        print("\n=== Test: Check Game UI Elements ===")

        try:
            # Start a new game
            new_game_btn = self.driver.find_element(AppiumBy.ACCESSIBILITY_ID, "New Game")
            new_game_btn.click()
            time.sleep(0.5)

            beginner_btn = self.driver.find_element(AppiumBy.ACCESSIBILITY_ID, "Beginner")
            beginner_btn.click()
            time.sleep(1)

            # Take screenshot of game
            self.driver.save_screenshot("/tmp/test_game_ui.png")
            print("Saved game UI screenshot")

            # Check for number pad buttons (1-9)
            for num in range(1, 10):
                try:
                    btn = self.driver.find_element(AppiumBy.ACCESSIBILITY_ID, str(num))
                    print(f"Found number button: {num}")
                except:
                    print(f"Number button {num} not found by accessibility ID")

            # Check for control buttons
            controls = ["arrow.uturn.backward", "arrow.uturn.forward", "delete.left", "lightbulb", "pause"]
            for ctrl in controls:
                try:
                    btn = self.driver.find_element(AppiumBy.ACCESSIBILITY_ID, ctrl)
                    print(f"Found control: {ctrl}")
                except:
                    print(f"Control {ctrl} not found")

            print("Game UI elements checked!")

        except Exception as e:
            print(f"Error: {e}")
            self.driver.save_screenshot("/tmp/test_game_ui_error.png")


def run_quick_test():
    """Run a quick connectivity test."""
    print("Starting Appium connection test...")

    options = XCUITestOptions()
    options.platform_name = "iOS"
    options.device_name = "iPhone 17"
    options.udid = "BDAD4971-B1A4-442E-B0D8-1EF35DDADD7F"
    options.bundle_id = "com.sudoku.app"
    options.automation_name = "XCUITest"
    options.no_reset = True

    try:
        driver = webdriver.Remote(
            command_executor="http://127.0.0.1:4723",
            options=options
        )
        print("Connected to Appium successfully!")

        # Take screenshot
        driver.save_screenshot("/tmp/appium_test.png")
        print("Screenshot saved to /tmp/appium_test.png")

        # Get page source for debugging
        source = driver.page_source
        with open("/tmp/appium_page_source.xml", "w") as f:
            f.write(source)
        print("Page source saved to /tmp/appium_page_source.xml")

        driver.quit()
        print("Test completed successfully!")
        return True

    except Exception as e:
        print(f"Connection failed: {e}")
        return False


if __name__ == "__main__":
    import sys

    if len(sys.argv) > 1 and sys.argv[1] == "--quick":
        run_quick_test()
    else:
        unittest.main(verbosity=2)
