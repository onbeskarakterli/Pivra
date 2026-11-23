"""Protocol and lifetime association controls. No native GPU/leak claims."""
import unittest
from gdb_report import outcome, ownership

class GdbReport(unittest.TestCase):
    def test_raw_255_without_inferior_result_is_not_app_crash(self):
        r=outcome(255,[dict(kind='observer_breakpoint',symbol='eglDestroyContext')])
        self.assertEqual((r['status'],r['application_status']),('BLOCKED','UNKNOWN'))

    def test_real_exit_and_signal_remain_distinct(self):
        r=outcome(0,[dict(kind='inferior_result',state='exited',exit_code=0)])
        self.assertEqual(r['status'],'PASS')
        r=outcome(134,[dict(kind='inferior_result',state='signal',signal='SIGABRT')])
        self.assertEqual((r['status'],r['application_status']),('FAIL','SIGNAL'))

    def test_mismatched_debugger_exit_is_blocked(self):
        r=outcome(255,[dict(kind='inferior_result',state='exited',exit_code=0)])
        self.assertEqual((r['status'],r['application_status']),('BLOCKED','PASS'))

    def test_signal_trap_is_unclassified_not_application_crash(self):
        r=outcome(125,[dict(kind='inferior_result',state='signal',signal='SIGTRAP')])
        self.assertEqual((r['status'],r['application_status']),('BLOCKED','UNKNOWN'))

    def test_incomplete_duplicate_or_malformed_protocol_cannot_pass(self):
        value=dict(kind='inferior_result',state='exited',exit_code=0)
        for records in ([value,value],[value,dict(kind='protocol_error')],[dict(kind='inferior_result',state='debugger_stopped')]):
            self.assertEqual(outcome(0,records)['status'],'BLOCKED')

    def event(self,kind,symbol,time,**fields):
        return dict(kind=kind,symbol=symbol,time_us=time,pid=1,display='0x100',**fields)

    def map_line(self,obj,time):
        return f'PIVRA_GPU_CONTEXT_OWNER object={obj} owner=0xd native_context=512 native_display=256 share_context=768 gdk_object=0xf platform=1 time_us={time} pid=1 tid=9'

    def test_handles_and_retirement_are_associations_not_leak_verdicts(self):
        events=[self.event('egl_return','eglCreateContext',10,return_value='0x200'),self.event('observer_breakpoint','eglDestroyContext',40,context='0x200',call_id=1)]
        log='\n'.join([self.map_line('0xa',20),'PIVRA_GPU_CONTEXT_RELEASE object=0xa refs=1 time_us=30 pid=1','PIVRA_GPU_LIFECYCLE stage=context-retired owner=0xa time_us=35 pid=1'])
        r=ownership(events,log)[0]
        self.assertEqual(r['owner_candidates'][0]['role'],'pivra-worker')
        self.assertTrue(r['owner_candidates'][0]['retirement_observed_before_request'])
        self.assertIsNone(r['request_return'])
        self.assertIn('EGL may defer destruction',r['caveat'])

    def test_reused_native_handle_does_not_select_old_owner(self):
        events=[self.event('egl_return','eglCreateContext',10,return_value='0x200'),self.event('egl_return','eglCreateContext',60,return_value='0x200'),self.event('observer_breakpoint','eglDestroyContext',90,context='0x200')]
        r=ownership(events,self.map_line('0xa',20)+'\n'+self.map_line('0xb',70))[0]
        self.assertEqual([e['object'] for e in r['owner_candidates']],['0xb'])

    def test_failed_termination_and_later_initialization_do_not_mean_terminated(self):
        request=self.event('observer_breakpoint','eglDestroyContext',50,context='0x200')
        failed=self.event('egl_return','eglTerminate',20,return_value=0)
        self.assertIsNone(ownership([failed,request],'')[0]['display_last_successful_transition'])
        success=self.event('egl_return','eglTerminate',20,return_value=1)
        init=self.event('egl_return','eglInitialize',30,return_value=1)
        self.assertEqual(ownership([success,init,request],'')[0]['display_last_successful_transition']['symbol'],'eglInitialize')

    def test_gdk_share_and_unknown_handles_are_not_assumed_worker_ownership(self):
        request=self.event('observer_breakpoint','eglDestroyContext',50,context='0x300')
        r=ownership([request],self.map_line('0xa',20))[0]
        self.assertEqual(r['owner_candidates'][0]['role'],'gdk-share-context')
        request['context']='0x999'
        self.assertEqual(ownership([request],self.map_line('0xa',20))[0]['owner_candidates'],[])

    def test_entry_only_termination_does_not_claim_display_was_terminated(self):
        events=[self.event('observer_breakpoint','eglTerminate',20),self.event('observer_breakpoint','eglDestroyContext',40,context='0x200')]
        r=ownership(events,'')[-1]
        self.assertEqual(r['display_last_request']['symbol'],'eglTerminate')
        self.assertIsNone(r['display_last_successful_transition'])
