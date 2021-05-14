pub struct Transition <T: PartialEq, EvtT: PartialEq+Copy, ManagedType: Sized>
{
    from:   T,
    to:     T,
    event:  EvtT,
    t_func: Option<Box<dyn FnMut(EvtT, &mut ManagedType) ->()>>
}

impl <'a, T: PartialEq, EvtT: PartialEq+Copy, ManagedType: Sized > Transition<T, EvtT, ManagedType>
{
    pub fn new(from_state: T, to_state: T, trigger: EvtT) -> Self
    {
        Transition
        {
            from: from_state,
            to: to_state,
            event: trigger, 
            t_func: None
        }
    }

    pub fn new_trig(from_state: T, to_state: T, trigger: EvtT, trigger_func: Box<dyn FnMut(EvtT, &mut ManagedType) -> ()>) -> Self
    {
        Transition
        {
            from: from_state,
            to: to_state,
            event: trigger, 
            t_func: Some(trigger_func)
        }
    }
}

pub struct FiniteStateMachine<StateType: PartialEq , EventType: PartialEq+Copy, ManagedType>
{
    current_state: StateType,
    managed_state: ManagedType,
    transitions: Vec<Transition<StateType, EventType, ManagedType>>
}

impl<StateType: PartialEq  + Copy, EventType: PartialEq+Copy, ManagedType> FiniteStateMachine<StateType, EventType, ManagedType>
{
    pub fn new(start_state: StateType, managed_state: ManagedType, transitions: Vec<Transition<StateType, EventType, ManagedType>>) -> Self
    {
        FiniteStateMachine {
            current_state: start_state,
            managed_state,
            transitions
        }
    }

    pub fn trigger_event(&mut self, e: EventType)
    {
        // check if a transition exists in the current state, that
        // matches the event:        
        let items: Vec<&Transition<StateType, EventType, ManagedType>> = self.transitions.iter().filter(|transition| transition.from == self.current_state 
                                                                                                     && std::mem::discriminant(&transition.event) == std::mem::discriminant(&e)).collect();                                                                                        
        if items.len() == 0
        {
            panic!("Bad trigger in current state!");
        }

        if items.len() != 1
        {
            panic!("Ambiguous trigger")
        }

        let state_to_find = self.current_state;
        let actual_transition = self.transitions.iter_mut().filter(|transition| transition.from == state_to_find
                                && std::mem::discriminant(&transition.event) == std::mem::discriminant(&e)).next().unwrap();
        self.current_state = actual_transition.to;

        if let Some(func) = &mut actual_transition.t_func
        {
            func(e, &mut self.managed_state);
        }
        
    }

    pub fn get_current_state(&self) -> StateType
    {
        self.current_state
    }

    pub fn get_managed_state(&self) -> &ManagedType
    {
        return &self.managed_state;
    }
}

#[cfg(test)]
mod tests
{
    use crate::fsm::*;

    #[derive(Debug, PartialEq, Clone, Copy)]
    enum MiniFsm{None, Idle, Busy}

    #[derive(Debug, Clone, Copy, PartialEq)]
    enum Evt{Step, ToIdle}

    fn create_fsm() -> FiniteStateMachine<MiniFsm, Evt, i32>
    {
        FiniteStateMachine::new(MiniFsm::None, 
            4,
            vec![
                Transition::new(MiniFsm::None, MiniFsm::Idle,  Evt::Step),
                Transition::new(MiniFsm::Idle, MiniFsm::Busy,  Evt::Step),
                Transition::new(MiniFsm::Busy, MiniFsm::Idle,  Evt::ToIdle)
            ])
    }

    #[test]
    fn can_create_simple_fsm()
    {
        let _fsm = create_fsm();
        assert!(true);
    }

    #[test]
    fn can_trigger_transition()
    {
        let mut fsm = create_fsm();
        
        fsm.trigger_event(Evt::Step);
        assert!(true);
    }

    #[test]
    #[should_panic]
    fn panics_if_bad_state_transition_is_triggerd()
    {
        let mut fsm = create_fsm();
        
        fsm.trigger_event(Evt::ToIdle);     

    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    enum PayloadEvent
    {
        NoData,
        SomeData(u32)
    }

    #[derive(Debug, PartialEq, Clone, Copy)]
    enum PayloadStates{
        StateA,
        StateB
    }

    #[test]
    fn calls_trigger_if_set()
    {
        let trig_fun = |_arg: PayloadEvent, managed_state: &mut i32| {
            *managed_state = 10; 
        };

        let trig_with_data = |arg: PayloadEvent, managed_state: &mut i32| {
            if let PayloadEvent::SomeData(i) = arg
            {
                *managed_state = i as i32;
            }
        };

        let t0 = Transition::new_trig(
            PayloadStates::StateA, 
            PayloadStates::StateB, 
            PayloadEvent::NoData, 
            Box::new(trig_fun));
        
            let t1 = Transition::new_trig(PayloadStates::StateA, PayloadStates::StateB, PayloadEvent::SomeData(0), Box::new(trig_with_data));

        let transitions = vec![t0,t1];

        let mut fsm = FiniteStateMachine::new(PayloadStates::StateA, 4,  transitions);
        fsm.trigger_event(PayloadEvent::NoData);
        assert_eq!(*fsm.get_managed_state(), 10);

        let transitions2 = vec![
            Transition::new(PayloadStates::StateA, PayloadStates::StateB, PayloadEvent::NoData),
            Transition::new_trig(PayloadStates::StateA, PayloadStates::StateB, PayloadEvent::SomeData(0),  Box::new(trig_with_data))
        ];


        let mut fsm = FiniteStateMachine::new(PayloadStates::StateA, 21, transitions2);
        fsm.trigger_event(PayloadEvent::SomeData(45));
        assert_eq!(*fsm.get_managed_state(), 45);
    }   
}