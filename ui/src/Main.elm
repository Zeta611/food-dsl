module Main exposing (main)

import Browser
import Draggable
import Html exposing (Html, aside, div, h2, input, label, node, span, text)
import Html.Attributes exposing (checked, class, property, step, style, type_, value)
import Html.Events exposing (onCheck, onInput)
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
    , height : Float
    , fluteAmplitude : Float
    , fluteCount : Int
    , ringAmplitude : Float
    , ringCount : Int
    }


type alias PanelState =
    { pos : ( Float, Float )
    , drag : Draggable.State ()
    }


init : () -> ( Model, Cmd Msg )
init _ =
    ( { cup =
            { bottomRadius = 10
            , topRadius = 14
            , height = 16
            , fluteAmplitude = 0.9
            , fluteCount = 16
            , ringAmplitude = 0
            , ringCount = 0
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
    | CupSetHeight Float
    | CupSetFluteAmplitude Float
    | CupSetFluteCount Int
    | CupSetRingAmplitude Float
    | CupSetRingCount Int
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
                [ h2 (class "card-title justify-between cursor-move" :: dragHandle)
                    [ text "Configuration"
                    , span [ class "icon-[hugeicons--grip-vertical]" ] []
                    ]
                , slider "Bottom radius" { min = 5, max = 40, step = 0.5 } cup.bottomRadius CupSetBottomRadius
                , slider "Top radius" { min = 5, max = 50, step = 0.5 } cup.topRadius CupSetTopRadius
                , slider "Height" { min = 5, max = 60, step = 1 } cup.height CupSetHeight

                -- min radius 5 - 2 - 2 > 0
                , slider "Flute depth" { min = 0, max = 2, step = 0.1 } cup.fluteAmplitude CupSetFluteAmplitude
                , slider "Flutes" { min = 0, max = 40, step = 1 } (toFloat cup.fluteCount) (round >> CupSetFluteCount)
                , slider "Ring depth" { min = 0, max = 2, step = 0.1 } cup.ringAmplitude CupSetRingAmplitude
                , slider "Rings" { min = 0, max = 20, step = 1 } (toFloat cup.ringCount) (round >> CupSetRingCount)
                , label [ class "flex justify-between text-sm" ]
                    [ text "Wireframe"
                    , input
                        [ type_ "checkbox"
                        , class "toggle toggle-sm"
                        , checked wireframe
                        , onCheck SetWireframe
                        ]
                        []
                    ]
                ]
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


{-| Must match `Cup` in `dsl/src/lib.rs`
-}
encodeCup : Cup -> E.Value
encodeCup cup =
    E.object
        [ ( "bottomRadius", E.float cup.bottomRadius )
        , ( "topRadius", E.float cup.topRadius )
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
        ]



-- SUBSCRIPTIONS


subscriptions : Model -> Sub Msg
subscriptions model =
    Draggable.subscriptions PanelDragMsg model.panelState.drag
