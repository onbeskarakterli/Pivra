import unittest
from slideshow_result import result


def evidence():
    lines = []
    for n in (1, 2, 3):
        lines.extend([f'SLIDESHOW_STAGE cycle={n} corrupt-select',
                      f'SLIDESHOW_STAGE cycle={n} video-selected',
                      'PIVRA_SLIDESHOW event=video-ended path="4.mp4"',
                      f'SLIDESHOW_STAGE cycle={n} photo-presented stock_opened={n} gpu_opened=0'])
    return '\n'.join(lines) + '\nPASS slideshow EOS regression: 3 corrupt/video/photo cycles and independent close\n'


class SlideshowResult(unittest.TestCase):
    def test_three_complete_cycles(self):
        self.assertEqual(result(0, evidence(), cycles=True)['status'], 'PASS')

    def test_signal_after_success_markers_still_fails(self):
        self.assertEqual(result(139, evidence(), cycles=True)['status'], 'FAIL')

    def test_glycin_prerequisite_is_not_run(self):
        self.assertEqual(result(77, '')['status'], 'NOT RUN')

    def test_corrupt_skip_without_eos_fails(self):
        log = evidence().replace('PIVRA_SLIDESHOW event=video-ended', 'error skip')
        self.assertEqual(result(0, log, cycles=True)['status'], 'FAIL')

    def test_selected_title_is_not_a_presented_photo(self):
        self.assertEqual(result(0, evidence().replace('photo-presented', 'photo-selected'), cycles=True)['status'], 'FAIL')

    def test_missing_middle_cycle_eos_fails(self):
        log = evidence().splitlines()
        del log[6]
        self.assertEqual(result(0, '\n'.join(log), cycles=True)['status'], 'FAIL')

    def test_incomplete_debugger_is_blocked(self):
        self.assertEqual(result(0, evidence(), cycles=True, debugger_records=[])['status'], 'BLOCKED')

    def test_verified_debugger_signal_fails(self):
        records = [dict(kind='inferior_result', state='signal', signal='SIGSEGV')]
        self.assertEqual(result(139, evidence(), cycles=True, debugger_records=records)['status'], 'FAIL')

    def test_verified_debugger_exit(self):
        records = [dict(kind='inferior_result', state='exited', exit_code=0)]
        self.assertEqual(result(0, evidence(), cycles=True, debugger_records=records)['status'], 'PASS')

    def test_context_warning_is_not_suppressed(self):
        for marker in ('gdk_gl_context_make_current() failed', 'No EGL configuration available',
                       'PIVRA_GPU_FAIL', 'PIVRA_GPU_SHADER_FAIL', 'PIVRA_GPU_CONTEXT_FAILURE'):
            with self.subTest(marker=marker):
                self.assertEqual(result(0, evidence() + marker, cycles=True)['status'], 'FAIL')

    def test_incomplete_close_fails(self):
        self.assertEqual(result(0, evidence().replace('PASS slideshow EOS regression:', 'unfinished:'), cycles=True)['status'], 'FAIL')


if __name__ == '__main__':
    unittest.main()
