require "minitest/autorun"

module UI
  def self.user_error!(message)
    raise ArgumentError, message
  end

  def self.success(_message); end
end

# Exercise the release lane without loading credentials or contacting Apple.
class SubmissionHarness
  attr_reader :lanes, :submissions

  def initialize
    @lanes = {}
    @submissions = []
    path = File.expand_path("../Fastfile", __dir__)
    instance_eval(File.read(path), path)
  end

  def default_platform(_platform); end
  def desc(*_description); end
  def before_all; end
  def error; end

  def platform(_platform)
    yield
  end

  def lane(name, &block)
    @lanes[name] = block
  end

  def upload_to_app_store(options)
    @submissions << options
  end
end

class SubmitReviewTest < Minitest::Test
  def setup
    @harness = SubmissionHarness.new
  end

  def test_selects_the_requested_build_and_requires_manual_release
    @harness.lanes[:submit_review].call(version: "1.1.4", build_number: "202610192200", release_notes: "Faster puzzles.")
    submission = @harness.submissions.fetch(0)
    assert_equal "1.1.4", submission[:app_version]
    assert_equal "202610192200", submission[:build_number]
    assert_equal true, submission[:submit_for_review]
    assert_equal false, submission[:automatic_release]
    assert_equal true, submission[:skip_binary_upload]
    assert_equal true, submission[:skip_screenshots]
    assert_equal({ "en-US" => "Faster puzzles." }, submission[:release_notes])
    assert_equal false, submission[:precheck_include_in_app_purchases]
  end

  def test_missing_version_or_build_cannot_select_an_unintended_latest_build
    [{}, { version: "1.1.4" }, { build_number: "202610192200" }].each do |options|
      assert_raises(ArgumentError) { @harness.lanes[:submit_review].call(options) }
    end
    assert_empty @harness.submissions
  end

  def test_invalid_identifiers_and_empty_release_notes_do_not_submit
    [
      { version: "latest", build_number: "1", release_notes: "Notes" },
      { version: "1.1.4", build_number: "latest", release_notes: "Notes" },
      { version: "1.1.4", build_number: "1", release_notes: " " }
    ].each do |options|
      assert_raises(ArgumentError) { @harness.lanes[:submit_review].call(options) }
    end
    assert_empty @harness.submissions
  end
end
