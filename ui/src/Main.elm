module Main exposing (main)

import Browser
import Draggable
import Html exposing (Html, aside, button, div, h2, input, label, node, span, text)
import Html.Attributes exposing (checked, class, classList, property, step, style, type_, value)
import Html.Events exposing (onCheck, onClick, onInput)
import Json.Encode as E


main : Program () Model Msg
main =
    Browser.element
        { init = init
        , view = view
        , update = update
        , subscriptions = subscriptions
        }



-- MODEL


type alias Model =
    { cup : Cup
    , wireframe : Bool
    , panelState : PanelState
    }


{-| Mockup "AST"
-}
type alias Cup =
    { bottomRadius : Float
    , topRadius : Float
    , bottom : Profile
    , top : Profile
    , height : Float
    , fluteAmplitude : Float
    , fluteCount : Int
    , ringAmplitude : Float
    , ringCount : Int
    , carved : Bool
    , wall : Float
    , floor : Float
    , cavityBottom : Profile
    , cavityTop : Profile
    }


{-| Must match `Profile` in `dsl/src/lib.rs`
-}
type Profile
    = Circle
    | Squircle


type alias PanelState =
    { pos : ( Float, Float )
    , drag : Draggable.State ()
    }


init : () -> ( Model, Cmd Msg )
init _ =
    ( { cup =
            { bottomRadius = 10
            , topRadius = 14
            , bottom = Circle
            , top = Circle
            , height = 16
            , fluteAmplitude = 0.7
            , fluteCount = 23
            , ringAmplitude = 0.5
            , ringCount = 1
            , carved = True
            , wall = 3.1
            , floor = 2
            , cavityBottom = Circle
            , cavityTop = Circle
            }
      , wireframe = False
      , panelState =
            { pos = ( 0, 0 )
            , drag = Draggable.init
            }
      }
    , Cmd.none
    )



-- UPDATE


type Msg
    = CupSetBottomRadius Float
    | CupSetTopRadius Float
    | CupSetBottom Profile
    | CupSetTop Profile
    | CupSetHeight Float
    | CupSetFluteAmplitude Float
    | CupSetFluteCount Int
    | CupSetRingAmplitude Float
    | CupSetRingCount Int
    | CupSetCarved Bool
    | CupSetWall Float
    | CupSetFloor Float
    | CupSetCavityBottom Profile
    | CupSetCavityTop Profile
    | SetWireframe Bool
    | PanelOnDragBy Draggable.Delta
    | PanelDragMsg (Draggable.Msg ())


update : Msg -> Model -> ( Model, Cmd Msg )
update msg ({ cup, panelState } as model) =
    case msg of
        CupSetBottomRadius r ->
            ( { model | cup = { cup | bottomRadius = r } }, Cmd.none )

        CupSetTopRadius r ->
            ( { model | cup = { cup | topRadius = r } }, Cmd.none )

        CupSetBottom p ->
            ( { model | cup = { cup | bottom = p } }, Cmd.none )

        CupSetTop p ->
            ( { model | cup = { cup | top = p } }, Cmd.none )

        CupSetHeight h ->
            ( { model | cup = { cup | height = h } }, Cmd.none )

        CupSetFluteAmplitude a ->
            ( { model | cup = { cup | fluteAmplitude = a } }, Cmd.none )

        CupSetFluteCount n ->
            ( { model | cup = { cup | fluteCount = n } }, Cmd.none )

        CupSetRingAmplitude a ->
            ( { model | cup = { cup | ringAmplitude = a } }, Cmd.none )

        CupSetRingCount n ->
            ( { model | cup = { cup | ringCount = n } }, Cmd.none )

        CupSetCarved c ->
            ( { model | cup = { cup | carved = c } }, Cmd.none )

        CupSetWall w ->
            ( { model | cup = { cup | wall = w } }, Cmd.none )

        CupSetFloor f ->
            ( { model | cup = { cup | floor = f } }, Cmd.none )

        CupSetCavityBottom p ->
            ( { model | cup = { cup | cavityBottom = p } }, Cmd.none )

        CupSetCavityTop p ->
            ( { model | cup = { cup | cavityTop = p } }, Cmd.none )

        SetWireframe w ->
            ( { model | wireframe = w }, Cmd.none )

        PanelOnDragBy ( dx, dy ) ->
            let
                ( x, y ) =
                    panelState.pos
            in
            ( { model
                | panelState =
                    { panelState
                        | pos = ( x + dx, y + dy )
                    }
              }
            , Cmd.none
            )

        PanelDragMsg dragMsg ->
            Tuple.mapFirst
                (\x -> { model | panelState = x })
                (Draggable.update dragConfig dragMsg panelState)


dragConfig : Draggable.Config () Msg
dragConfig =
    Draggable.basicConfig PanelOnDragBy



-- VIEW


view : Model -> Html Msg
view { cup, wireframe, panelState } =
    let
        ( x, y ) =
            panelState.pos

        panelTrans =
            "translate(" ++ String.fromFloat x ++ "px," ++ String.fromFloat y ++ "px)"

        dragHandle =
            Draggable.mouseTrigger () PanelDragMsg :: Draggable.touchTriggers () PanelDragMsg
    in
    div [ class "relative h-screen w-screen overflow-hidden bg-base-200" ]
        [ node "render-preview"
            [ class "absolute inset-0 block"
            , property "cup" (encodeCup cup)
            , property "wireframe" (E.bool wireframe)
            ]
            []
        , aside
            [ class
                "card card-sm absolute top-4 right-4 w-80 bg-base-100 border-base-300 border-2 text-base-content shadow-lg select-none resize-y overflow-auto"
            , style "transform" panelTrans
            ]
            [ div [ class "card-body gap-2" ]
                ([ h2 (class "card-title justify-between cursor-move" :: dragHandle)
                    [ text "Configuration"
                    , span [ class "icon-[hugeicons--grip-vertical]" ] []
                    ]
                 , profilePicker "Bottom" cup.bottom CupSetBottom
                 , slider "Bottom radius" { min = 5, max = 40, step = 0.5 } cup.bottomRadius CupSetBottomRadius
                 , profilePicker "Top" cup.top CupSetTop
                 , slider "Top radius" { min = 5, max = 50, step = 0.5 } cup.topRadius CupSetTopRadius
                 , slider "Height" { min = 5, max = 60, step = 1 } cup.height CupSetHeight

                 -- min radius 5 - 2 - 2 > 0
                 , slider "Flute depth" { min = 0, max = 2, step = 0.1 } cup.fluteAmplitude CupSetFluteAmplitude
                 , slider "Flutes" { min = 0, max = 40, step = 1 } (toFloat cup.fluteCount) (round >> CupSetFluteCount)
                 , slider "Ring depth" { min = 0, max = 2, step = 0.1 } cup.ringAmplitude CupSetRingAmplitude
                 , slider "Rings" { min = 0, max = 20, step = 1 } (toFloat cup.ringCount) (round >> CupSetRingCount)
                 , toggle "Carved" cup.carved CupSetCarved
                 ]
                    ++ (if cup.carved then
                            [ -- cavity radius = radius - wall - flute depth - ring depth
                              slider "Wall" { min = 0.5, max = 4, step = 0.1 } cup.wall CupSetWall
                            , slider "Floor" { min = 0.5, max = 5, step = 0.5 } cup.floor CupSetFloor
                            , profilePicker "Inside bottom" cup.cavityBottom CupSetCavityBottom
                            , profilePicker "Inside top" cup.cavityTop CupSetCavityTop
                            ]

                        else
                            []
                       )
                    ++ [ toggle "Wireframe" wireframe SetWireframe ]
                )
            ]
        ]


slider :
    String
    -> { min : Float, max : Float, step : Float }
    -> Float
    -> (Float -> Msg)
    -> Html Msg
slider name range current toMsg =
    label [ class "flex flex-col gap-1" ]
        [ span [ class "flex justify-between text-sm" ]
            [ text name
            , span [ class "text-base-content/70" ] [ text (String.fromFloat current) ]
            ]
        , input
            [ type_ "range"
            , class "range range-xs"
            , Html.Attributes.min (String.fromFloat range.min)
            , Html.Attributes.max (String.fromFloat range.max)
            , step (String.fromFloat range.step)
            , value (String.fromFloat current)
            , onInput (String.toFloat >> Maybe.withDefault current >> toMsg)
            ]
            []
        ]


toggle : String -> Bool -> (Bool -> Msg) -> Html Msg
toggle name current toMsg =
    label [ class "flex justify-between text-sm" ]
        [ text name
        , input
            [ type_ "checkbox"
            , class "toggle toggle-sm"
            , checked current
            , onCheck toMsg
            ]
            []
        ]


profilePicker : String -> Profile -> (Profile -> Msg) -> Html Msg
profilePicker name current toMsg =
    let
        option profile caption =
            button
                [ type_ "button"
                , class "btn btn-xs join-item"
                , classList [ ( "btn-active", profile == current ) ]
                , onClick (toMsg profile)
                ]
                [ text caption ]
    in
    div [ class "flex items-center justify-between text-sm" ]
        [ text name
        , div [ class "join" ]
            [ option Circle "Circle"
            , option Squircle "Squircle"
            ]
        ]


{-| Must match `Cup` in `dsl/src/lib.rs`
-}
encodeCup : Cup -> E.Value
encodeCup cup =
    E.object
        [ ( "bottomRadius", E.float cup.bottomRadius )
        , ( "topRadius", E.float cup.topRadius )
        , ( "bottom", encodeProfile cup.bottom )
        , ( "top", encodeProfile cup.top )
        , ( "height", E.float cup.height )
        , ( "flutes"
          , E.object
                [ ( "amplitude", E.float cup.fluteAmplitude )
                , ( "count", E.int cup.fluteCount )
                ]
          )
        , ( "rings"
          , E.object
                [ ( "amplitude", E.float cup.ringAmplitude )
                , ( "count", E.int cup.ringCount )
                ]
          )
        , ( "cavity"
          , if cup.carved then
                E.object
                    [ ( "bottom", encodeProfile cup.cavityBottom )
                    , ( "top", encodeProfile cup.cavityTop )
                    , ( "wall", E.float cup.wall )
                    , ( "floor", E.float cup.floor )
                    ]

            else
                E.null
          )
        ]


encodeProfile : Profile -> E.Value
encodeProfile profile =
    case profile of
        Circle ->
            E.string "circle"

        Squircle ->
            E.string "squircle"



-- SUBSCRIPTIONS


subscriptions : Model -> Sub Msg
subscriptions model =
    Draggable.subscriptions PanelDragMsg model.panelState.drag
