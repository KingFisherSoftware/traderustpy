import pytest
import traderusty


def test_sum_as_string():
    assert traderusty.sum_as_string(1, 1) == "2"
