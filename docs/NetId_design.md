NetId Design
1. NetId는 고유id를 가지고 접근 가능하다(NetId)
2. NetId 안에는 Input, Output Byte 배열 메모리를 가지고 있다.
3. I/O 메모리 접근 방식 2가지 지원한다.
    1) 물리적 주소 방식 : InOut, PhyAddress, Offset
    2) IO 번호 방식<Hash, Vec> : InOut, IoNumber
    3) Digital I/O BitPositon 필요
    4) Analog I/O 자료형 필요(u8, i8 ~ u32, i32, f32, u64, i64, f64) + 자료형에 Size가 포함되어 있음

4. Motor
    1) fn =>  power_on, power_off servo_on, servo_off, move_at, move_at_intime, rotate, emergency_stop, decel_Stop, error_set, error_reset
    2) 상태 머신

       ```mermaid
       stateDiagram-v2
           [*] --> PowerOff

           PowerOff --> PowerOn : power_on
           PowerOn --> PowerOff : power_off

           state PowerOn {
               [*] --> Stopped
               Stopped --> Moving : rotate / move_at / move_at_intime
               Moving --> Stopped : 이동 완료 / 정지

               state Moving {
                   [*] --> Acceleration
                   Acceleration --> ConstantSpeed : 목표 속도 도달
                   ConstantSpeed --> Deceleration : 정지 또는 감속 명령
                   Acceleration --> Deceleration : 정지 또는 감속 명령
                   Deceleration --> [*] : 속도 0
               }

               Stopped --> Error : 오류 발생
               Moving --> Error : 오류 발생
               Error --> Stopped : error_reset [오류 해제 가능]
           }

           PowerOn --> PowerOff : power_off
           PowerOff --> PowerOff : emergency_stop
           PowerOn --> PowerOn : emergency_stop / 즉시 정지 후 안전 상태 유지
       ```

       `ConstantSpeed`는 가속이 끝난 뒤 목표 속도를 유지하는 정속 운전 상태이다. `Decelerating`을 거쳐 속도가 0이 되면 `Stopped`로 전이한다. `decel_Stop`은 감속 정지를 요청하고, `emergency_stop`은 일반 감속 정지와 구분되는 즉시 정지 요청이다. 실제 비상 정지 후 전원 및 오류 상태 처리는 드라이버/하드웨어 정책에 맞춘다.
